
// disk
struct DiskIO {

}

impl DiskIO {
    // queue a request to fetch data
    fn begin_request() {}

    // cancel a request (if possible)
    fn cancel_request() {}

    // returns the completion state of a request that was previously made
    fn take_completed_request() {}
}

// an IO queue that handles not overloading requests?


// asset storage that handles a committed and latest version of an asset (that is still being built)
struct AssetStorage {

}

impl AssetStorage {
    // given data, starts the process of preparing the data to be used in-engine (maybe including uploading to GPU)
    fn prepare_asset() {}
    // commits the specified loading data to be visible to the rest of the game
    fn commit_asset() {}
    // drop the asset and release resources associated with it
    fn free_asset() {}
}

// hot reload manager that can pause streamer requests and inject requests that would be reloading existing assets that changed

// Streaming Manager that looks at all requests and determines what can fit in memory and requests it
struct LoadRequestManager {

}

impl LoadRequestManager {
    fn new_request(); // TODO: Return an object (LoadHandle) and have the below functions on it
    fn update_request();
    fn destroy_request();
    //Rename: add_ref(), remove_ref()
}

//
// Streaming frontend that can also create handles based on camera position, visibility, etc.
//
struct StreamingLoader {

}

impl StreamingLoader {
    fn new_view(); // TODO: Return an object and have the below functions on it
    fn update_view();
    fn destroy_view();
}

//
// Frontend that allows creating handles
//
struct GameLoader {

}

impl GameLoader {
    fn request_asset(); // TODO: Return an object and have the below functions on it
    // fn update_request();
    // fn destroy_request();
}
