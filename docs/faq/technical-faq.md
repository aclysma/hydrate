# Technical FAQ

Like most complex systems, hydrate is full of engineering trade-offs. I want to record
some of those choices and the rationale for them

## Q: Why path-based Assets? Why ID-based Assets?

Most asset systems will either try to mirror a file system, or mirror a database.
In a file system, you have a hierarchy of named files, whereas a database is usually 
a flat list of objects identified by an immutable ID.

An ID-based system does not necessarily imply a database. Storing assets on disk
using an immutable ID as the filename can get many of the advantages while still
being compatible with typical source control systems. Further, using a hash of the
contents as the filename offers interesting guarantees of consistency, even when
using a file system. (See .git/objects/* in any git repo for an example)

I initially wanted to use strictly an ID-based approach. Renaming or moving files
is problematic; references by path break easily, and it requires iterating over 
all files (or having them all loaded) to detect that a rename will break a reference.
Additionally, renaming or moving a folder affects all files under that folder. This
can make move operations enormously more complicated and slow for the asset system
and other infrastructure (like source control) that it relies on.

This paper describes many of these problems, and an interesting solution to handling
them [perforce-aaa-source-control.pdf](perforce-aaa-source-control.pdf) My initial 
solution closely mirrored this, using immutable IDs for assets and "fake"
directories represented by a file. A rename or move of a folder in this case only
requires changing the "parent" field within a single file.

Finally, I anticipate having a huge number of small assets that are authored
in-engine, such as placements in a level editor. Having to think about names/locations
for these files seems like an unnecessary burden.

If all content was built in-engine, a pure ID-based approach would likely work 
very well. However, the reality is that many of the source files an asset system
should understand use paths extensively. For example, a GLTF file may reference
a texture by filename on disk. Path-based references in source data is a common case.

In the end, I implemented both. A project can contain multiple asset sources, and
assets can be moved freely between the source types. I recommend in-engine authored
content use the ID-based storage, and content closely associated with source files
be stored in Path-based storage. All assets, even assets stored in path-based asset
sources have stable IDs.

## Q: Why are properties stored as flattened string keys?

In order to support inheritance of property data (i.e. prototypes), we need to:
 - distinguish between "unset" and "set". Not having a particular property path
   as a key means "use the prototype's value, or the default value"
 - read the value of this property on potentially many assets. A single lookup
   in a single hashmap is preferable to pointer-chasing through multiple hashmaps
   per object

## Q: Why are .meta files necessary/What are they used for?

These files permanently associate content in a file with a stable ID. This means
a re-import will update existing content rather than make another copy.

## Q: What is an importable, and how are importable names chosen?

An importable is any unit of data in a source file that can be imported as an asset.
The importers *scans* a file to see what importables are available, and then may
import one or more of them.

The names are chosen by the importer of that filetype. There is not significance to
them other than 1) the importer should return a consistent name for each unit of
data between multiple imports 2) the name should ideally be helpful to users to
understand what unit of data they are importing.

## Q: Can I edit asset files with a text editor? Why/Why not.

I originally wanted asset files to be friendly to hand-editing in a text editor.
While simple tweaks are still pretty safe (like changing a numeric value) I strongly
suggest NOT adding/removing properties, mainly because of how schema migration works.

Assets are stored alongside the schema that describes their format. The name of the
properties needs to be consistent with that schema, not the current version of the
schema used by the project. This is because schema migration is lazy. In other words,
changing the schema in a project does not immediately require a migration of existing
content.

## Q: What data should be checked into source control?

 - Check in source code and original assets (i.e. .psd, .blend, etc.)
 - Probably check in exported data (i.e. .png, .tif, .gltf, etc.)
 - Always check in asset files.
 - Import data should almost always be checked in. The exception is that if all
   assets are stored in path-based asset sources and no assets are manually
   imported, checking in import data is optional. Because in this case, import
   data is regenerated whenever the asset source is loaded.
 - Most likely, do not check in build data.

For large teams, it may not be necessary for all users to sync all data. For
example, programmers likely do not need to sync down original assets.

While asset files can likely be merged by hand most of the time, it is simplest
to use source control to lock the files and avoid any possibility of merge conflicts.

## Q: Why is schema migration lazy? Why are schemas stored in assets?

Imagine the following scenario:
 - User A creates an asset with schema version 1
 - User B modifies the schema to version 2 and commits it to source control
 - User A pulls latest from source control. They now have an asset that does not
   match. There was no way for the migration to be done ahead of time because user B
   never had user A's asset locally and could not have known about it

Or another scenario:
 - Project A contains some asset, with some very old schema
 - Project B wants to re-use that asset, and it has a much newer version of
   that schema. The asset is copied or integrated over to the new project.

(This scenario is similar to what would happen if the asset was in some kind
of store like UE Marketplace or Unity Asset Store.)

By having schema stored in assets, and making extensive use of UUIDs, all of these
scenarios can be handled gracefully with no loss of user data.

## Q: Why is schema stored in JSON instead of code?

There are lots of advantages:
 - Storing schema data in source code strongly ties the asset system to a
   particular language. I think it would be very reasonable to support importers
   written in python or a shipping game runtime in C/C++
 - Encoding schema in rust code would likely involve complicated procmacros that
   are hard to maintain and have compile overhead
 - If schema is just data, then it can be changed without recompiling tools.
 - There are other minor advantages, like needing to support schema as data for
   schema migration, being able to pick exactly what kinds of types are supported
   rather than having to support whatever rust supports, etc.

## Q: To what degree is this project production ready/What level of stability and support is offered?

The primary goal in open-sourcing this project is to share *ideas*, not code. If
the code is helpful to you, great, but that is not the primary goal. I would love
to see other projects learn from this project and make something even better.

My plan is to add/maintain the features that I need for my future demos and
experiments. It's unlikely I will add features suggested by others that I'm not
interested in using personally.


