# Schema Code Generation

`hydrate-codegen` can be used to generate rust code to interact with schema data in a typesafe way.

## Command Line Options

```
USAGE:
    hydrate-codegen [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
        --trace      
    -V, --version    Prints version information

OPTIONS:
        --included-schema <included-schema>...    
        --job-name <job-name>                     
        --outfile <outfile>                       
        --schema-path <schema-path>   
```

There are two general ways the tool can be used.
 - The recommended way is to add code generation jobs to the hydrate_project.json. Then, the tool can run with no arguments (meaning do all jobs in the hydrate_project.json) or with `--job-name` referencing a particular job in hydrate_project.json.
	 - Options that can be supplied via command line can be defined in the project configuration for convenience.
 - Or options can be directly supplied via command line

The command line interface:

 - `--job-name`: Run the named job, as defined in hydrate_project.json. Generally, use this option OR the below options.
 - `--schema-path`: The path to the folder containing schemas to generate code for
 - `--included-schema`: If a schema in `schema-path` references a schema stored elsewhere, the directory containing it must be supplied here
 - `--outfile`: Rust code will be emitted to this file

## Using Generated Code

It is recommended to set the `--outfile` to some rust file within a crate that then includes that file via `include!()`. The generated code will reference some hydrate types. You may need to provide the required imports like so:

```rust
use hydrate_model::{DataContainer, DataContainerRef, DataContainerRefMut, DataSetResult};  
use std::cell::RefCell;  
use std::rc::Rc;  
  
include!("generated.rs");
```

The generated code provides four kinds of wrappers for each record type:
 - `Record`: An "owned" record
 - `RecordRef`: Read-only reference to a record
 - `RecordRefMut`: A read-write reference to a record
 - `RecordAccessor`: Does not contain the record or a reference to the record. Instead, the data must be provided with every function call to the accessor to read or mutate that data. This is rarely used.

Most likely, you will either be creating a `Record` or using a `RecordRef` or `RecordRefMut`. Here is an example of creating a record in a custom importer:

```rust
impl Importer for TransformImporter {
	fn import_file(  
	    &self,  
	    context: ImportContext,  
	) -> PipelineResult<()> {
		let json_str = std::fs::read_to_string(context.path)?;  
		let json_data: TransformJsonFileFormat = serde_json::from_str(&json_str)?;
		  
		let transform = TransformRecord::new_builder(context.schema_set);
		transform.position().set_vec3(json_data.position)?;
		transform.rotation().set_vec4(json_data.rotation)?;
		transform.scale().set_vec3(json_data.scale)?;
		  
		context.add_default_importable(transform.into_inner()?, None);
	}
}
```

Note that the methods on a `Record` may return additional `Record` or `Field` helper objects that eventually result in setting property values. 