# Adam's Personal OpenGraph image generator

This is a Rust project that generates custom OpenGraph preview images for every post on my personal blog at [127.io](https://127.io/).  It's not intended for direct use by anyone else, but I broke it out into a separate (public) repo so that I could more easily share the code as part of a blog post about how to generate OpenGraph images in Rust in general, and using Cloudflare Functions and WASM in particular.

If you want to use this for your own site, you should defintely read that post first, and then fork this repo and customize it to your needs.  Please do not just rip off my OpenGraph image template without at least changing the site name to be yours, to avoid confusing people and pissing me off.

# Usage Hints

The way I use this repo is as a Git submodule in my (private) Git repo that contains my blog content.  You can add this as a submodule to your repo this way:

```shell
git submodule add https://github.com/anelson/og-generator.git
```

This will add the directory `og-generator` which contains the contents of the `master` branch.

Your repo is assumed to contain a Cloudflare Pages project at the root.  You need to make a `functions/` directory if you don't already have one, and create a file there called `og-image.ts` with the following contents:

```typescript
// The meat of the implemenation is part of the `og-generator` submodule.
// Just re-export that here
import { onRequestGet } from "../og-generator/src/og-image";

export { onRequestGet };
```

To compile the og-generator WASM, in the `og-generator` submodule directory run `just wasm-cfp` (this assumes you have installed the Just tool) to compile the Rust code to a WASM module.

Before publishing, you need to generate a JSON file with a list of all posts on your site and the metadata that is used to generate the OpenGraph images.  It should look something like this:

```json
{
    "default": { "title": "Creative Articulation", "description": "Musings and misadventures of an expat enterpreneur" },
	"/2024/11/16/generating-opengraph-image-cards-for-my-zola-static-blog-with-cloudflare-functions-and-rust/": {
            "title": "Generating OpenGraph image cards for my Zola static blog with Cloudflare Functions and Rust",
	    "description":
	    
		"I didn't like how plain the links to my blog posts rendered on 𝕏 and in Slack.  So I set about implementing OpenGraph image cards for each post, the hard way, using Cloudflare Functions and Rust.\n"
	    
	},
	"/2024/11/01/whats-in-my-air-raid-bug-out-bag/": {
            "title": "What's in my air raid bug-out bag?",
	    "description":
	    
		"I went overboard prepping for a very specific threat scenario, specifically: large-scale Russian drone or missile attacks on Kyiv.  The result is this bug-out bag.\n"
	    
	}
}
```

and should be stored in `public/og-metadata.xml` (relative to your repo root, not the submodule root).  Yes, it's a JSON file but the extension is `.xml`.  I warned you that this was for my own personal use!

Assuming you did all of that correctly, at your Cloudflare Pages project URL you can append `/og-image?p=whatever` where `whatever` is a field in the JSON object contained in `og-metadata.xml`, and you will get back a PNG suitable for use as the OpenGraph preview image for that post.


