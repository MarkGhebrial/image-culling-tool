# `image-culling-tool`

Absolutely no LLMs or generative AI were used in the creation of this project.

This is an application built on top of `eframe` that allows you to quickly cull a folder full of images.

## Goals:
- Be fast.
  - How?
    - Loading images from disk in parallel using `rayon`.
    - Holding all thumbnails in memory so that they never have to be fetched again
      from disk.
    - Caching the most recently used full-resolution images.
    - Preloading (from disk) the images most likely to be viewed next (the ones that are adjacent to the currently selected image).
- Have a keyboard focused control scheme.

## Intended workflow:
- `cd folder-of-images`, then `cull .` to open the GUI.
- Use the GUI to assign a star rating to each image
- Use the GUI to export images above a certain star rating to a folder