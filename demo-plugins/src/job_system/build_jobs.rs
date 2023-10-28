
// A build job should have a single output associated with the build job's ID (the ID may be the sha256 hash of job inputs)
// - For example, a job should be able to start job A and job B, and make B a dependent of A
// - job A might produce multiple things, start multiple other jobs, etc. But we don't know that until the job runs,
//   and B needs to be scheduled to run and able to access that later
// Another flow example:
//   - Some prerequisite work needed (optimizing vertex buffers)
//   - Multiple jobs take the work and format it different ways (a job to make position-only data, a job to make index data, a job to make full vertex data, etc.)
//   - A mesh object that needs all of these for the material it plans to use might be kicking it off
//
// Feel like a two-phase structure could work:
// - Enumerate Dependencies (allows requesting arbitrary data to be ready to use when running)
// - Run the job (allowed to fire off subjobs, they get memo-ized)
// - Finalize the job (allowed to read results of created jobs)
// - So if JobA kicks of JobB, JobB kicks of JobC:
//   - JobA: enumerate, run
//   - JobB: enumerate, run
//   - JobC: enumerate, run
//   - JobC: finalize
//   - JobB: finalize
//   - JobA: finalize
//
// One issue is we need a job that hasn't started yet to affect our job's output. Options:
// - We have to return a reference/handle/other form of indirection. We generate handle and child job uses it
// - We have to create an empty object that the child job populates.
//   - We can mark these "promises" as failed
//   - Maps well to the async mindset
//   - How do we handle memo-izing? The child job might be triggered with same input by multiple jobs that all
//     want to create the object to be filled by the job.
// - We have a second pass after the child jobs runs that can take the results of the child job and use them
//   to write the current job's output
//   - How to handle memo-ization. Could we end up with many copies of something?
//     - Yes if we have no way of optionally referencing rather than copying
//   - Might still benefit by passing a promise for an ID that points at some intermediate data, just so
//     we have fine-grained control of dependenies and can get good parallelization
//
// So subjobs should probably be able to create blobs of data referenced by UUID
//
// Could we omit having both run/finalize for simple jobs? Conceptually the run/finalize pair are two
// separate jobs.
//
// We could treat this like having signals/semaphores/promises?
//
// Jobs can create unfulfilled promises and pass them to other jobs
// - If the ID is deterministic based on inputs, we avoid memo-ization challanges
// The child jobs can signal the promises (which also means the data in it has been produced and is available)
// Jobs waiting for promises end up being unblocked



// trait BuildJobContext {
//     fn produce_intermediate_data(&mut self, )
//     fn produce_built_asset(&mut self, built_asset: BuiltAsset);
// }






use std::hash::Hash;
use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use siphasher::sip128::Hasher128;
use type_uuid::{Bytes, TypeUuid};
use uuid::Uuid;
use hydrate_base::hashing::HashMap;
use hydrate_base::ObjectId;
use hydrate_data::{DataSet, SchemaSet, SingleObject};
use hydrate_model::BuiltAsset;











