# Authoring Schemas

Any data stored by hydrate must match the form of a defined schemas. Schemas may be described programmatically or with json files. This doc will describe the format of json schema files.

When hydrate tools load, they will search for a hydrate_project.json, which has a list of locations with schema files to load.

## JSON Schema Files

Below is a rough outline of what a schema file looks like. Note that the file is an array of JSON objects. The objects may either be records or enums.

 - Records contain fields, like a struct. Enums contain symbols.
 - Enums are more similar to simple C-like enums that complex rust enums that may contain fields within an enum variant. 

```json
[  
  {  
    "type": "record",
    "name": "Vec3",
    "uuid": "ba0cab6d-3b47-4a65-b7e4-827763d038d9",
    "fields": [
      {
        "name": "x",
        "type": "f32",
        "uuid": "63ef086b-ced2-46d4-bea1-4afc8dfd99f0"
      }
    ]
  },
  {  
    "type": "enum",
    "name": "ShadowMethod",
    "uuid": "e00e0b54-c48d-4b22-96f0-df7edd6d0728",
    "symbols": [
      {
        "name": "None",
        "uuid": "22b99f65-ff07-43f7-9896-1f1694987f58"
      },
    ]
  }
]
```
## Records

The following values are allowed for a field:

 - `type`: indicates that the json object describes a record
 - `name`: An arbitrary name for the record. It should be globally unique and will be the way a schema might reference another schema.
 - `uuid`: Should be assigned a random UUID and never changed
 - `aliases`: For convenience, schemas may have multiple additional names. It is not necessary to add the old name of a record in the alias when the record is renamed.
 - `fields`: The fields that make up the record. See below for details
 - `display_name`: A name that will be used in the UI
 - `default_thumbnail`: A path to an image that will be used as a thumbnail for that particular kind of asset
 - `tags`: Used to flag records, can be used for example to get all records that have a particular tag

### Record Fields

 - `type`: indicates the type contained in this field. It could be another record, an enum, or built-in types such as numeric values or dynamic arrays. See below for more detail
 - `name`: An arbitrary name for the field. It should be unique within the record.
 - `uuid`: Should be assigned a random UUID and never changed
 - `aliases`: For convenience, schemas may have multiple additional names. It is not necessary to add the old name of a record in the alias when the record is renamed.
 - `display_name`: A friendly name that is shown in the property editor when editing this field
 - `category`: The property editor groups fields by categories
 - `description`: The property editor will show this text in the property editor as a "?" that can be moused over for a tooltip.
 - `ui_min`/`ui_max:` Defines a range of numbers that the UI should encourage but not enforce. Values outside this range would be considered "allowed" but unusual.
 - `clamp_min`/`clamp_max`: Defines a range of numbers that are allowed. Data stored with numbers outside the range should at least produce a warning and be clamped.
### Supported Field Types

 - `[Schema Name]`: The name of a user-defined record or enum. These fields will be by-value, not by-reference.
 - `bool`
 - `i32`, `i64`
 - `u32`, `u64`
 - `f32`, `f64`,
 - `bytes`: A byte array of arbitrary length (although large byte arrays, for example >1KB should be stored in import data, not assets.)
 - `string`: A string of arbitrary length
 - `nullable`: A value that can be set as null or non-null. Values within a nullable field that is non-null are presumed to be default values (0, "", etc.)
	 - `inner_type`: The type contained within the nullable
 - `static_array`: A container of elements of a single type. Each element is "keyed" by their index in the array. In other words, an example property override `positions.3.x` will always refer to `positions[3].x`. Array indexes are 0-based.
	 - `inner_type`: The type of the elements contained in the array
	 - `length`: The length of the array. Elements not assigned a value are presumed to be default values (0, "", etc.)
 - `dynamic_array`: A container of elements of a single type. Each element is "keyed" by a UUID. In other words, an example property override `positions.63ef086b-ced2-46d4-bea1-4afc8dfd99f0.x` will always refer to that particular element.
	 - `inner_type`: The type of the elements contained in the array
	 - **NOTE**: This UUID-keyed data inheritance is designed with data inheritance scenarios in mind. For example, if user A authors a dynamic array with several elements, user B overrides a field on the element, and user A re-orders the array, the edit from user B should follow the element with matching UUID, not simply be applied to the Nth element.
 - `map`: An associative container of keys/values of single types.
	 - `key_type`: The type used as a key for the container. It must be one of the following types: `bool`, `i32`, `i64`, `u32`, `u64`, `string`, `asset_ref`, or `enum`
	 - `value_type`: The type used as a value for the container.
	 - **NOTE**: This UUID-keyed data inheritance is designed with data inheritance scenarios in mind. For example, if user A authors a map with several key/value pairs, user B overrides a value on a particular pair, and user A modifies the key of an existing key/value pair, the property override from user B should still be applied to the newly-renamed key/value pair.
 - `asset_ref`: A reference to another asset. May be empty.
	 - `inner_type`: The expected type of the referenced asset. This is currently not enforced. It must be another record.

## Enums

Enums are similar to C-style enums where they can be one of several named options. These options are called "symbols." The following values are allowed for an enum:

 - `name`: An arbitrary name for the enum. It should be globally unique and will be the way a schema might reference another schema.
 - `uuid`: Should be assigned a random UUID and never changed
 - `aliases`: For convenience, schemas may have multiple additional names. It is not necessary to add the old name of an enum in the alias when the enum is renamed.
 - `symbols`: A list of symbols that correspond to the named options the enum represents
### Enum Symbols

The following values are allowed for enum symbols

 - `name`: An arbitrary name for the symbol. It should be unique within the enum. Case sensitive!
 - `uuid`: Should be assigned a random UUID and never changed
 - `aliases`: For convenience, schemas may have multiple additional names. It is not necessary to add the old name of an enum in the alias when the enum is renamed.


