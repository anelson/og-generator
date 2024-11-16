// Cloudflare Functions entry point for generating Open Graph images for blog posts.
//
// Calls into the `og-generator` WASM module to generate the image.

import { PagesFunction, PagesContext } from "@cloudflare/workers-types";

// Capture the og-metadata.xml file generated as part of the Zola build, as a "text" module
// so that we can refer to it to look up metadata for the post whose OG image we're generating.
//
// Despite the misleading ".html" extension, this is JSON.  But Cloudflare's
// text module support recognizes only HTML extensions for some reason.
import og_metadata_json from "../../public/og-metadata.html";

// Use `just wasm-dev` to build the WASM file before deploying this or testing it locally
import wasmModule from "../pkg/og_generator_bg.wasm";
import * as wasmBindings from "../pkg/og_generator";

// Instantiate the WASM module.
// It imports some wasm-bindgen generated Javascript helpers so those must be provided as imports.
const instance = await WebAssembly.instantiate(wasmModule, {
  "./og_generator_bg.js": wasmBindings,
});

// Set the WASM instance in the Javascript bindings, since the peculiarities of Cloudflare Workers
// are such that the generated wrappers can't do this themselves.
//
// See https://developers.cloudflare.com/workers/languages/rust/#javascript-plumbing-wasm-bindgen
// for at least some acknowledgement of this.
wasmBindings.__wbg_set_wasm(instance.exports);

// Parse the JSON OG metadata
const og_metadata: MetadataJson = JSON.parse(og_metadata_json);

// The OG image cards unfortunately take up to 2 seconds to generate, presumably because Cloudflare Functions
// aggressively limit the amount of CPU resources available to a given invocation.
//
// So we want to use a long cache duration to avoid regenerating the image every time it's requested.
//
// Note that separately, when we're serving the OG images to clients, we set a max-age of 1 hour which 
// ensures that clients will check back in w/ the server often enough to pick up changes to the post metadata.
//
// Since almost all of the time these requests will be served from this KV cache, there's no performance issue
// with having such a short max-age.
const CACHE_DURATION_SECONDS = 60 * 60 * 24 * 365; // 365 days

interface Env {
  CACHE: KVNamespace;
  // ... other binding types
}

interface PageMetadata {
  title: string;
  description: string;
}

interface MetadataJson {
  [key: string]: PageMetadata;
}

export const onRequestGet: PagesFunction<Env> = async (context) => {
  try {
    // Get the full path for the current request, from the `p` query string parameter.
    const url = new URL(context.request.url);
    const params = url.searchParams;
    // Get the value of the 'p' parameter
    const path = params.get("p");
    console.log("Post path:", path);

    if (!path) {
      throw new Error("Missing 'p' query parameter");
    }

    const imageBytes = await generate_og_image(context, path);

    // Return the image as a PNG
    return new Response(imageBytes, {
      headers: {
        "Content-Type": "image/png",

        // Cache the image for 1 hour.  The KV cache is longer than that, but we want clients
        // to re-request this content after an hour in case the post metadata has changed, meaning
        // the image itself changed.
        "Cache-Control": "public, max-age=3600, immutable",
      },
    });
  } catch (error) {
    console.error("Error generating OG image:", error);
    return new Response(`Error generating OG image: ${error.message}`, {
      status: 500,
    });
  }
};

export async function generate_og_image(
  context: PagesContext<Env>,
  path: string,
): Promise<Uint8Array> {
  // As the key, use the path for which we're looking up post metadata, plus a hash
  // of the metadata itself.  This way, if the metadata changes, we'll generate a new
  // cache key and thus a new image.
  let cacheKey: string;
  let postMetadata: PageMetadata;

  if (og_metadata[path]) {
    postMetadata = og_metadata[path];
    cacheKey = path;
  } else {
    postMetadata = og_metadata["default"];
    cacheKey = "default";
  }

  // Create a hash of the metadata using SHA-256
  const metadataHash = fnv1a(JSON.stringify(postMetadata));

  // Combine the path and metadata hash for the final cache key
  const finalCacheKey = `${cacheKey}:${metadataHash}`;

  console.log("Cache key:", finalCacheKey);

  const cachedImageBytes = await context.env.CACHE.get(
    finalCacheKey,
    "arrayBuffer",
  );

  if (cachedImageBytes) {
    console.log("Found cached OG image");
    return new Uint8Array(cachedImageBytes);
  }

  console.log("Generating OG image for ", finalCacheKey);

  console.log("Calling WASM generate_og_image");
  const imageBytes = wasmBindings.generate_og_image(
    "127.io | Creative Articulation",
    postMetadata.title,
    postMetadata.description.replace(/\s+/g, " ").trim(),
  );
  console.log("OG image generated successfully");

  await context.env.CACHE.put(finalCacheKey, imageBytes, {
    expirationTtl: CACHE_DURATION_SECONDS,
  });

  return imageBytes;
}

// FNV-1a hash function
// Generated by Claude Sonnet  3.5, hopefully it's not a hallucination...
function fnv1a(str: string): string {
  let hash = 0x811c9dc5; // 32-bit FNV offset basis
  for (let i = 0; i < str.length; i++) {
    hash ^= str.charCodeAt(i);
    hash +=
      (hash << 1) + (hash << 4) + (hash << 7) + (hash << 8) + (hash << 24);
  }
  return (hash >>> 0).toString(16); // Convert to 8-character hex string
}
