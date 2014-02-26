//
// zhtta.rs
//
// Starting code for PS3
// Running on Rust 0.9
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
// 
// University of Virginia - cs4414 Spring 2014
// Weilin Xu and David Evans
// Version 0.5

// To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="zhtta=debug"

#[feature(globs)];
extern mod extra;

use std::io::*;
use std::io::net::ip::{SocketAddr};
use std::{os, str, libc, from_str};
use std::path::Path;
use std::hashmap::HashMap;

use extra::getopts;
use extra::arc::MutexArc;
use extra::priority_queue::PriorityQueue;

static SERVER_NAME : &'static str = "Zhtta Version 0.5";

static IP : &'static str = "127.0.0.1";
static PORT : uint = 4414;
static WWW_DIR : &'static str = "./www";

static HTTP_OK : &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n";
static HTTP_BAD : &'static str = "HTTP/1.1 404 Not Found\r\n\r\n";

static COUNTER_STYLE : &'static str = "<doctype !html><html><head><title>Hello, Rust!</title>
             <style>body { background-color: #884414; color: #FFEEAA}
                    h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red }
                    h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green }
             </style></head>
             <body>";
static TASKS: uint = 12;

mod gash;

struct HTTP_Request {
    // Use peer_name as the key to access TcpStream in hashmap. 

    // (Due to a bug in extra::arc in Rust 0.9, it is very inconvenient to use TcpStream without the "Freeze" bound.
    //  See issue: https://github.com/mozilla/rust/issues/12139)
    peer_name: ~str,
    path: ~Path,
}

struct QueuedRequest {
  priority: uint,
  request: HTTP_Request,
}

impl Ord for QueuedRequest {
  fn lt(&self, other: &QueuedRequest) -> bool {self.priority < other.priority}
}

struct WebServer {
    ip: ~str,
    port: uint,
    www_dir_path: ~Path,

    tasks: uint,
    
    request_queue_arc: MutexArc<~PriorityQueue<QueuedRequest>>,
    stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>,

    notify_port: Port<()>,
    shared_notify_chan: SharedChan<()>,
    
    visitor_arc: MutexArc<uint>,
}

impl WebServer {
    fn new(ip: &str, port: uint, www_dir: &str, tasks: uint) -> WebServer {
        let (notify_port, shared_notify_chan) = SharedChan::new();
        let www_dir_path = ~Path::new(www_dir);
        os::change_dir(www_dir_path.clone());

        WebServer {
            ip: ip.to_owned(),
            port: port,
            www_dir_path: www_dir_path,

            tasks: tasks,
                        
            request_queue_arc: MutexArc::new(~PriorityQueue::<QueuedRequest>::new()),
            stream_map_arc: MutexArc::new(HashMap::new()),

            notify_port: notify_port,
            shared_notify_chan: shared_notify_chan,
	    
	    visitor_arc : MutexArc::new(0),
        }
    }
    
    fn run(&mut self) {
        self.listen();
        self.dequeue_static_file_request();
    }
    
    fn listen(&mut self) {
        let addr = from_str::<SocketAddr>(format!("{:s}:{:u}", self.ip, self.port)).expect("Address error.");
        let www_dir_path_str = self.www_dir_path.as_str().expect("invalid www path?").to_owned();
        
        let request_queue_arc = self.request_queue_arc.clone();
        let shared_notify_chan = self.shared_notify_chan.clone();
        let stream_map_arc = self.stream_map_arc.clone();
		let visitor_arc = self.visitor_arc.clone();
                
        spawn(proc() {
            let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
            println!("{:s} listening on {:s} (serving from: {:s}).", 
                     SERVER_NAME, addr.to_str(), www_dir_path_str);
            
            for stream in acceptor.incoming() {
                let (queue_port, queue_chan) = Chan::new();
                queue_chan.send(request_queue_arc.clone());
                
                let notify_chan = shared_notify_chan.clone();
                let stream_map_arc = stream_map_arc.clone();
                
		let (count_port, count_chan) = Chan::new();
                count_chan.send(visitor_arc.clone());
		
                // Spawn a task to handle the connection.
                spawn(proc() {
                    let visitor_arc = count_port.recv();
                    visitor_arc.access(|visitor_count| *visitor_count += 1); //Fixed unsafe counter
                    let request_queue_arc = queue_port.recv();
                  
                    let mut stream = stream;
                    
                    let peer_name = WebServer::get_peer_name(&mut stream);
                    
                    let mut buf = [0, ..500];
                    stream.read(buf);
                    let request_str = str::from_utf8(buf);
                    debug!("Request:\n{:s}", request_str);
                    
                    let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
                    if req_group.len() > 2 {
                        let path_str = "." + req_group[1].to_owned();
                        
                        let mut path_obj = ~os::getcwd();
                        path_obj.push(path_str.clone());
                        
                        let ext_str = match path_obj.extension_str() {
                            Some(e) => e,
                            None => "",
                        };
                        
                        debug!("Requested path: [{:s}]", path_obj.as_str().expect("error"));
                        debug!("Requested path: [{:s}]", path_str);
                             
                        if path_str == ~"./" {
                            debug!("===== Counter Page request =====");
                            WebServer::respond_with_counter_page(visitor_arc, stream);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        } else if !path_obj.exists() || path_obj.is_dir() {
                            debug!("===== Error page request =====");
                            WebServer::respond_with_error_page(stream, path_obj);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        } else if ext_str == "shtml" { // Dynamic web pages.
                            debug!("===== Dynamic Page request =====");
                            WebServer::respond_with_dynamic_page(stream, path_obj);
                            debug!("=====Terminated connection from [{:s}].=====", peer_name);
                        } else { 
                            debug!("===== Static Page request =====");
                            WebServer::enqueue_static_file_request(stream, path_obj, stream_map_arc, request_queue_arc, notify_chan);
                        }
                    }
                });
            }
        });
    }

    fn respond_with_error_page(stream: Option<std::io::net::tcp::TcpStream>, path: &Path) {
        let mut stream = stream;
        let msg: ~str = format!("Cannot open: {:s}", path.as_str().expect("invalid path").to_owned());

        stream.write(HTTP_BAD.as_bytes());
        stream.write(msg.as_bytes());
    }

    // TODO: Safe visitor counter.
    fn respond_with_counter_page(visitor_arc: MutexArc<uint>, stream: Option<std::io::net::tcp::TcpStream>) {
        let mut stream = stream;
	visitor_arc.access(|visitor_count| //Fixed unsafe counter
        {
	    
	    let response: ~str = 
		format!("{:s}{:s}<h1>Greetings, Krusty!</h1>
			<h2>Visitor count: {:u}</h2></body></html>\r\n", 
			HTTP_OK, COUNTER_STYLE, 
			*visitor_count );
	    debug!("Responding to counter request");
	    stream.write(response.as_bytes());
	})
    }
    
    fn stream_static_file(stream: &mut Option<std::io::net::tcp::TcpStream>,
                  path: &Path) {
        stream.write(HTTP_OK.as_bytes());
        let read_count = 5242880; // Number of bytes to read at a time
        let mut reader = File::open(path).expect("Invalid file!");
        let mut error = None;
        debug!("Starting to read a file.");
        while error.is_none() {
            // Error becomes Some(_) when we reach EOF.
            let bytes = io_error::cond.trap(|e: IoError| error = Some(e))
                    .inside(|| reader.read_bytes(read_count));
            stream.write(bytes);
            debug!("Read {:u} bytes from file.", read_count);
        }
    }

    fn respond_with_dynamic_page(stream: Option<std::io::net::tcp::TcpStream>, path: &Path) {
        let mut stream = stream;
        let mut file_reader = File::open(path).expect("Invalid file!");
        stream.write(HTTP_OK.as_bytes());

        let begin_comment : ~[u8] = "<!--#exec cmd=\"".bytes().collect();
        let end_comment : ~[u8] = "\" -->".bytes().collect();

        // Stores bytes when a possible comment is encountered. Writes them
        // to the stream iff they are not a server-side exec command.
        let mut buffer: ~[u8] = ~[]; 
        // Stores the command to run in gash, if we think we found one.
        let mut cmd: ~[u8] = ~[];
        // The position of the buffer, relative to the current `begin_comment`
        // or `end_comment` byte arrays that we are parsing.
        let mut buffer_pos: uint = 0;
        // Whether we are parsing the beginning of a comment or not.
        let mut l_comment = false;
        // Whether we are parsing the end of a comment or not.
        let mut r_comment = false;

        // Do this byte-by-byte. Should be fast, but by golly is it ugly.
        for byte in file_reader.bytes() {
            // We haven't found anything that looks like a comment.
            if l_comment==false {
                if byte != begin_comment[0] {
                    // This is not the beginning of a comment. Write byte to
                    // the stream.
                    stream.write(&[byte]);
                }
                else {
                    // This is a comment. Add the byte to the possible comment
                    // buffer, and try to parse it as a comment.
                    l_comment = true;
                    // Add the byte to the buffer.
                    buffer = ~[byte];
                    // Buffer pos is 1 because we already added the first byte.
                    buffer_pos = 1;
                }
            }
            else {
                // We are parsing either a command, or the beginning of (what
                // we think is) an exec comment.
                if !r_comment {
                    // Whether we are at the end of a "begin comment" keyword
                    // or not.
                    let end_of_l = buffer_pos >= begin_comment.len();

                    // If we're not at the end of an opening comment, and this
                    // byte looks like the next character in an opening comment,
                    // then push the byte to the buffer.
                    if !end_of_l && byte == begin_comment[buffer_pos] {
                        buffer.push(byte);
                        buffer_pos += 1;
                    }
                    // Otherwise, if we made it all the way to the end of a
                    // "begin comment" without failing, then this is a command.
                    else if end_of_l && byte != end_comment[0] {
                        // Also buffer the byte, in case this turns out to not
                        // be a properly formatted comment and we want to send
                        // it to the client.
                        buffer.push(byte);
                        cmd.push(byte);
                    }
                    // This byte is the identifier for the end of a comment.
                    else if byte == end_comment[0] {
                        // Buffer the byte anyway, in case this turns out to
                        // not actually be an exec command.
                        buffer.push(byte);
                        r_comment = true;
                        // Buffer_pos is 1 because we already know this is an
                        // end comment and we already buffered the first byte
                        // for an end comment.
                        buffer_pos = 1;
                    }
                    // We didn't make it to the end of `begin_comment` without
                    // failing. Write the buffer to the stream, flush the
                    // buffers, and reset the comment flag.
                    else {
                        // Push the byte to the buffer, so we send it along
                        // with the rest of the bytes.
                        buffer.push(byte);
                        stream.write(buffer);

                        l_comment = false;
                        buffer_pos = 0;
                        cmd = ~[];
                        buffer = ~[];
                    }
                }
                // We successfully parsed a `begin_comment`, a command, and
                // the first character of an `end_comment`.
                else {
                    let end_of_r = buffer_pos >= end_comment.len();
                    // We're not at the end of `end_comment`, and this byte
                    // still looks like an `end_comment`.
                    if !end_of_r && byte == end_comment[buffer_pos] {
                        buffer.push(byte);
                        buffer_pos += 1;
                    }
                    // We made it to the end of `end_comment` successfully,
                    // and we got a command. Run it in gash, then write it
                    // to the stream and flush the buffers.
                    else if end_of_r {
                        let result = gash::run_cmdline(std::str::from_utf8(cmd));
                        stream.write(result.as_bytes());
                        stream.write(&[byte]);

                        r_comment = false;
                        l_comment = false;
                        buffer_pos = 0;
                        buffer = ~[];
                        cmd = ~[];
                    }
                    // We didn't make it to the end of `end_comment` without
                    // failing, so this is not a server-side gash command.
                    // Write the buffer to the stream, then flush the buffers.
                    else {
                        // Push the current byte first, so it isn't lost.
                        buffer.push(byte);
                        stream.write(buffer);

                        r_comment = false;
                        l_comment = false;
                        buffer_pos = 0;
                        buffer = ~[];
                        cmd = ~[];
                    }
                }
            }
        }
    }
    
    // TODO: Smarter Scheduling.
    fn enqueue_static_file_request(stream: Option<std::io::net::tcp::TcpStream>,
                                   path_obj: &Path, stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>,
                                   req_queue_arc: MutexArc<~PriorityQueue<QueuedRequest>>,
                                   notify_chan: SharedChan<()>) {
        // Save stream in hashmap for later response.
        let mut stream = stream;
        let peer_name = WebServer::get_peer_name(&mut stream);
        let (stream_port, stream_chan) = Chan::new();
        stream_chan.send(stream);
        unsafe {
            // Use an unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            stream_map_arc.unsafe_access(|local_stream_map| {
                let stream = stream_port.recv();
                local_stream_map.swap(peer_name.clone(), stream);
            });
        }
        
        // Enqueue the HTTP request.
        let req = HTTP_Request { peer_name: peer_name.clone(), path: ~path_obj.clone() };
        let (req_port, req_chan) = Chan::new();
        req_chan.send(req);

        debug!("Waiting for queue mutex lock.");
        req_queue_arc.access(|local_req_queue| {
            debug!("Got queue mutex lock.");
            let req: HTTP_Request = req_port.recv();
            let mut prior = 0;
            
            let ip1 : &str = "128.143.";
            let ip2 : &str = "137.54.";
            
			if peer_name.slice_to(8).eq(&ip1) || peer_name.slice_to(7).eq(&ip2) {
				prior = 9;
			}
			
			let size_prior = WebServer::get_size_priority(path_obj);
			prior += size_prior;
			
            
            local_req_queue.push( QueuedRequest {priority: prior, request: req});//Change the priority depending on #3 and #5
            debug!("A new request enqueued, now the length of queue is {:u}.", local_req_queue.len());
        });
        
        notify_chan.send(()); // Send incoming notification to responder task.
    }

    fn spawn_semaphore(completion_port: Port<bool>, 
                       new_task_chan: Chan<MutexArc<HashMap<~Path, ~[u8]>>>,
                       max_tasks: uint) {
        spawn(proc() {
            // Counter for the number of response tasks active.
            let mut response_tasks = 0;
            // Dummy variable for static cache implementation
            let static_cache: MutexArc<HashMap<Path, ~[u8]>>;
            let static_cache = MutexArc::new(HashMap::new());
            loop {
                match completion_port.try_recv() {
                    Some(_) => {
                        response_tasks -= 1;
                        debug!("{:u} tasks are running.", response_tasks);
                    }
                    None => { }
                };
                if response_tasks < max_tasks {
                    // send a copy of the cache
                    if new_task_chan.try_send(static_cache.clone()) {
                        // There was a new task waiting in the queue.
                        response_tasks += 1;
                        debug!("New task started! {:u} tasks are running.",
                               response_tasks);
                    };
                }
                std::io::timer::sleep(100);
            }
        });
    }
    
    // TODO: Smarter Scheduling.
    fn dequeue_static_file_request(&mut self) {
        let req_queue_get = self.request_queue_arc.clone();
        let stream_map_get = self.stream_map_arc.clone();
        
        // Port<> cannot be sent to another task. So we have to make this task as the main task that can access self.notify_port.
        let (request_port, request_chan) = Chan::new();

        let (completion_port, completion_chan) = SharedChan::new();
        let (new_task_port, new_task_chan) = Chan::new();
        WebServer::spawn_semaphore(completion_port, new_task_chan, self.tasks);

        loop {
            self.notify_port.recv();    // waiting for new request enqueued.
            new_task_port.recv();

            req_queue_get.access( |req_queue| {
                match req_queue.maybe_pop() { // Priority Queue, pops off whatever has greatest value
                    None => { /* do nothing */ }
                    Some(req) => {
                        request_chan.send(req);
                        debug!("A new request dequeued, now the length of queue is {:u}.", req_queue.len());
                    }
                }
            });
            
            let request = request_port.recv().request;

            // Get stream from hashmap.
            // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
            let (stream_port, stream_chan) = Chan::new();
            unsafe {
                stream_map_get.unsafe_access(|local_stream_map| {
                    let stream = local_stream_map.pop(&request.peer_name).expect("no option tcpstream");
                    stream_chan.send(stream);
                });
            }

            let completion_chan = completion_chan.clone();
            spawn(proc() {
                let mut stream = stream_port.recv();
                debug!("Spawning static file transfer task.");
                let path = request.path.clone();
                debug!("=====Terminated connection from [{:s}].=====", request.peer_name);
                WebServer::stream_static_file(&mut stream, path);
                // Close stream automatically.
                completion_chan.send(true);
            });
        }
    }
    
    fn get_peer_name(stream: &mut Option<std::io::net::tcp::TcpStream>) -> ~str {
        match *stream {
            Some(ref mut s) => {
                         match s.peer_name() {
                            Some(pn) => {pn.to_str()},
                            None => (~"")
                         }
                       },
            None => (~"")
        }
    }
    
    // Returns the size priority based on the file size in bytes. Smaller files are given a higher priority, larger files a lower priority.
    fn get_size_priority(path: &Path) -> uint {
		let file = path.stat();
		let size = file.size;
		if (size < 5120) { return 8; }		// 5K
		if (size < 5242880) { return 7;	}	// 5M
		if (size < 10485760) { return 6; }	// 10M
		if (size < 20971520) { return 5; }	// 20M
		if (size < 41943040) { return 4; }	// 40M
		if (size < 83886080) { return 3; }	// 80M
		if (size < 536870912) {	return 2; }	// 512M
		return 1;

    }
}

fn get_args() -> (~str, uint, ~str, uint) {
    fn print_usage(program: &str) {
        println!("Usage: {:s} [options]", program);
        println!("--ip     \tIP address, \"{:s}\" by default.", IP);
        println!("--port   \tport number, \"{:u}\" by default.", PORT);
        println!("--tasks  \tmax tasks, \"{:u}\" by default.", TASKS);
        println!("--www    \tworking directory, \"{:s}\" by default", WWW_DIR);
        println("-h --help \tUsage");
    }
    
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    let program = args[0].clone();
    
    let opts = ~[
        getopts::optopt("ip"),
        getopts::optopt("port"),
        getopts::optopt("tasks"),
        getopts::optopt("www"),
        getopts::optflag("h"),
        getopts::optflag("help")
    ];

    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };

    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program);
        unsafe { libc::exit(1); }
    }
    
    let ip_str = if matches.opt_present("ip") {
                    matches.opt_str("ip").expect("invalid ip address?").to_owned()
                 } else {
                    IP.to_owned()
                 };
    
    let port:uint = if matches.opt_present("port") {
                        from_str::from_str(matches.opt_str("port").expect("invalid port number?")).expect("not uint?")
                    } else {
                        PORT
                    };
    
    let www_dir_str = if matches.opt_present("www") {
                        matches.opt_str("www").expect("invalid www argument?") 
                      } else { WWW_DIR.to_owned() };

    let tasks:uint = if matches.opt_present("tasks") {
                        from_str::from_str(matches.opt_str("tasks").expect("invalid tasks number?")).expect("not uint?")
                    } else {
                        TASKS
                    };
    debug!("Using {:u} max response tasks.", tasks);
    (ip_str, port, www_dir_str, tasks)
}

fn main() {
    let (ip_str, port, www_dir_str, tasks) = get_args();
    let mut zhtta = WebServer::new(ip_str, port, www_dir_str, tasks);
    zhtta.run();
}
