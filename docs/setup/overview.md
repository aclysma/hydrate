# What is Hydrate?

Hydrate is a content authoring framework and asset pipeline for games, reading artist-friendly formats from disk, processing them into your engine-ready formats, and delivering them to your game runtime. Hydrate handles large, complex datasets (Many thousands of assets, GBs of data) robustly and quickly.

## Content Authoring Framework

Hydrate supports in-engine authored data (for example, engine-specific materials, texture compression settings, node graphs) as well as imported data (png files, gltf files, etc.). The provided editor can be used to create new assets directly or import from existing source files. Some features:

 - Editor with asset browser and property editor with undo/redo
 - Identify assets with UUIDs or Paths - your choice
 - Define your own data types:
	 - Asset types
	 - Source file formats
	 - Build steps and final built data formats
 - Schema is defined in data, not code.
	 - Schema and game logic changes don't require recompiling the editor
	 - Hydrate can work well with non-Rust games!
 - Schema migration for when data types change
 - On-disk format for assets is mostly readable/diffable plain text
## Asset Pipeline

Once data has been authored or imported, it needs to be transformed into a runtime-friendly format. Custom build steps can read asset or imported data, as well as intermediate output of other build steps. 

- Parallel execution across cores
- Caching mechanisms to avoid duplicated work, even across multiple imports and builds
- On-demand loading of import data to scale to data sets larger than system memory
- Respects dependencies between assets and build jobs
- Surfaces warnings/errors in editor UI

## Loading/Game Runtime

Hydrate provides a reference loader in Rust:

 - FAST. Large, multi-GB scenes load in less than a second.
 - Automatic dependency loading
 - Hot reload of data
 - Extension points to support GPU upload and other custom load scenarios