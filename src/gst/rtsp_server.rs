use glib;
use gstreamer;
use gstreamer_rtsp_server;
use gstreamer_rtsp_server::prelude::*;

#[derive(Clone, Debug)]
pub struct RTSPServer {
    pub pipeline: String,
    port: u16,
    event_loop: glib::MainLoop,
}

impl Default for RTSPServer {
    fn default() -> Self {
        RTSPServer {
            pipeline: "videotestsrc ! video/x-raw,width=640,height=480 ! videoconvert ! x264enc ! rtph264pay name=pay0".to_string(),
            port: 8554,
            event_loop: glib::MainLoop::new(None, false),
        }
    }
}

impl RTSPServer {
    pub fn set_pipeline(&mut self, pipeline: &str) {
        self.pipeline = String::from(pipeline);
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = port;
    }

    pub fn run_loop(&self) {
        self.stop();
        // Start or restart the pipeline
        RTSPServer::rtsp_loop(self);
    }

    pub fn stop(&self) {
        // If there is an event loop for this pipeline, we are going to kill it with fire
        println!("self.event_loop: {}", self.event_loop.is_running());
        if self.event_loop.is_running() {
            self.event_loop.quit();
        }
    }

    fn rtsp_loop(rtsp_server: &RTSPServer) {
        match gstreamer::init() {
            Ok(_) => {}
            Err(error) => {
                println!("Error! {}", error);
                std::process::exit(-1);
            }
        }

        let server = gstreamer_rtsp_server::RTSPServer::new();
        server.set_service(&rtsp_server.port.to_string());

        // Much like HTTP servers, RTSP servers have multiple endpoints that
        // provide different streams. Here, we ask our server to give
        // us a reference to his list of endpoints, so we can add our
        // test endpoint, providing the pipeline from the cli.
        let mounts = server
            .get_mount_points()
            .ok_or("Could not get mount points")
            .unwrap();

        // Next, we create a factory for the endpoint we want to create.
        // The job of the factory is to create a new pipeline for each client that
        // connects, or (if configured to do so) to reuse an existing pipeline.
        let factory = gstreamer_rtsp_server::RTSPMediaFactory::new();

        // Here we tell the media factory the media we want to serve.
        // This is done in the launch syntax. When the first client connects,
        // the factory will use this syntax to create a new pipeline instance.
        factory.set_launch(&rtsp_server.pipeline);

        // This setting specifies whether each connecting client gets the output
        // of a new instance of the pipeline, or whether all connected clients share
        // the output of the same pipeline.
        // If you want to stream a fixed video you have stored on the server to any
        // client, you would not set this to shared here (since every client wants
        // to start at the beginning of the video). But if you want to distribute
        // a live source, you will probably want to set this to shared, to save
        // computing and memory capacity on the server.
        factory.set_shared(true);

        // Now we add a new mount-point and tell the RTSP server to serve the content
        // provided by the factory we configured above, when a client connects to
        // this specific path.
        mounts.add_factory("/video1", &factory);

        // Attach the server to our main context.
        // A main context is the thing where other stuff is registering itself for its
        // events (e.g. sockets, GStreamer bus, ...) and the main loop is something that
        // polls the main context for its events and dispatches them to whoever is
        // interested in them. In this example, we only do have one, so we can
        // leave the context parameter empty, it will automatically select
        // the default one.
        let id = server.attach(None);

        // Start the mainloop. From this point on, the server will start to serve
        // our quality content to connecting clients.
        rtsp_server.event_loop.run();

        glib::source_remove(id);

        // Get all clients that are connected and close the connection
        for client in server.client_filter(None) {
            client.close();
        }
    }
}
