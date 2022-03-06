use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::process::exit;
use std::str::FromStr;
use std::thread;
use std::time;
use std::io::BufWriter;


#[allow(dead_code)]
#[derive(Debug)]
struct PHeader {
    magic: (u8, u8),
    len: (u8, u8),
    msg: (u8, u8),
    other: Option<Vec<u8>>,
}

impl PHeader {
    fn string(&self) -> String {
        format!(
            "<PHeader MAGIC={:X?} LEN={:X?} MSG={:X?} HAS_EXT=({})>",
            self.magic,
            self.len,
            self.msg,
            self.other.is_some()
        )
    }
    /*
        fn get_ext_payload(&self) -> Result<Vec<u8>, bool> {
            if self.other.is_some() {
                Ok(self.other.to_owned().unwrap())
            } else {
                Err(false)
            }
        }
    */
}


// Takes an array of u8 arrays; returns PHeader object

fn bytes_to_header(input: &[u8]) -> Result<PHeader, bool> {
    let mut _temp: Vec<u8> = vec![0_u8; 8];
    if input.len() >= 8 {
        _temp = input[..8].to_vec();
        let _magic = &input[..2];
        let _len = &input[2..4];
        let _idk = &input[4..6];
        let _msg = &input[6..8];

        Ok(PHeader {
            magic: (_magic[0], _magic[1]),
            len: (_len[0], _len[1]),
            msg: (_msg[0], _msg[1]),
            other: if input.len() > 8 {
                Some(input[8..].to_owned())
            } else {
                None
            },
        })
    } else {
        Err(false)
    }
}


// Returns login payload to get into the the camera
fn login_payload() -> Vec<u8> {
    let username_bytes: Vec<u8> = "admin".as_bytes().to_vec();
    let password_bytes: Vec<u8> = "12345".as_bytes().to_vec();
    let mut header: Vec<u8> = vec![171, 205, 0, 128, 0, 0, 1, 16];
    let mut payload: Vec<u8> = vec![0; 128];
    payload.splice(0..username_bytes.len(), username_bytes);
    payload.splice(64..64 + password_bytes.len(), password_bytes);
    header.extend_from_slice(&payload);

    header
}

fn handle_rtp_client(mut stream : TcpStream){
    let t = time::Duration::from_secs(3);
    loop{
        
        //let mut payload : Vec<u8> = Vec::new();
        //stream.read_(&mut payload).unwrap();
        let mut payload = String::new();
        stream.read_to_string(&mut payload).unwrap();
        //println!("{:?}" , payload);
        //let reqs : Vec<String> = payload.split(' ').into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
        
        let reqs : Vec<String> = payload.split_terminator(&['\r','\n']).filter(|s| !s.is_empty()).map(|s| s.trim().to_string()).collect();
        //println!("{:?}->{}" , reqs , reqs.len());
        if reqs.len() == 3{
            
            let req_head : Vec<String> = reqs[0].split_whitespace().map(|s| s.trim().to_string()).collect();
            let req_cseq : Vec<String> = reqs[1].split(":").map(|s| s.trim().to_string()).collect();
            let req_ua : Vec<String> = reqs[2].split(":").map(|s| s.trim().to_string()).collect();
            println!("=>> {:?}\n{:?}\n{:?} <<=" , req_head , req_cseq , req_ua); 
            if req_head[0] == "OPTIONS"{
                
                println!("got options");
                stream.write_all(b"RTSP/1.0 200 OK\r\nCSeq: 2\r\nPublic: DESCRIBE, SETUP, PLAY, PAUSE, RECORD\r\n\r\n").unwrap();
                //stream.write(b"CSeq: 2\r\n").unwrap();
                //stream.write(b"Public: DESCRIBE, SETUP, PLAY, PAUSE, RECORD\r\n\r\n").unwrap();

            }
                
        }
        
        /*
        for item in payload.split_whitespace(){
            println!("{:?}" , item );
        }
        */
        /*
        if reqs.len() > 1{

        println!("{:?}" , reqs);
        let req_type = &reqs[0];

        println!("Method => {}" , req_type);
        
        if req_type == "OPTIONS"{
            
            println!("got options");
            stream.write(&get_rtsp_status(200, "OK")).unwrap();
            let desc = String::from("Public: DESCRIBE, SETUP, PLAY, PAUSE, RECORD\r\n\r\n").as_bytes().to_vec();
            stream.write(&desc).expect("Failed to write public desc.");
        }
        }
        */

        //thread::sleep(t);

    }

}


fn get_rtsp_status(code : u16 , status_msg : &str) -> Vec<u8>{
    
    let packet = format!("RTSP/1.0 {} {}\r\n" , code , status_msg);
    let bts = packet.as_bytes();
    bts.to_vec()
}

// Connects to camera and dumps raw headless h264 data into video.h264 file
fn rtsp_thread() {
    let feed_stream = UdpSocket::bind("[::]:6669").unwrap();
    feed_stream.connect("192.168.100.1:6669").unwrap();
    let mut vid = std::fs::File::create("video.h264").unwrap();
    let mut seqnum : u16 = 0; // sequence number; increase with packet with '2'
    let mut elpased : u32 = 0; // elapsed time of frames; u32 integer; 
    let mut video_data : Vec<u8> = Vec::new(); //raw headless h264 live video data
    let mut network_packet : Vec<u8> = Vec::new(); //fill packet to send to the client
    let mut fff = std::fs::File::create("xcal.h264").unwrap();
    //let vids = UdpSocket::bind("[::]:8557").unwrap();
    //vids.connect("[::]:8555").unwrap();
    //vids.set_broadcast(true).unwrap();
    //vids.send(b"hello").unwrap();

    /*for stm in rtp_server.incoming(){
        
        match stm {
            
            Ok(s) => { thread::spawn(move|| handle_rtp_client(s)); },
            Err(_) => { println!("Connection failed to new client"); break; }

        }

    };*/


    loop {
        let mut buf = [0_u8; 50]; //header buffer to peek
        feed_stream.peek_from(&mut buf).unwrap();
        let header = &buf[..8]; // stripped down version of header
        let mag = &header[..2]; //magic number; should be xBCDE
        let _size = &header[2..4];
        let size = u16::from_be_bytes([_size[0], _size[1]]); // 16bit size of payload
 
        //let seqn = &header[4..6];
        let msg = &header[6..8]; //message type; '1' for video, '2' for elpased data

        let mut payload: Vec<u8> = vec![0_u8; size as usize + 8];

        let _ = feed_stream.recv_from(&mut payload).unwrap();
        
        
        

        if msg[1] == 1 && mag == [188, 222] { //raw video data 
            
            video_data = payload[8..].to_vec();
        //    let raw_vid = &payload[8..];
            fff.write(&video_data).unwrap();
            /*
            println!(
                "MAGIC={:?} | SIZE={:?} | SEQ={:X?} | MSG={:X?}",
                &mag, &size, &seqn, &msg
            );*/
        }
            
        if msg[1] == 2 && mag == [188,222]{
            //network_packet.push(128);
            //network_packet.push(99);

            //fff.write_all(&video_data).unwrap();
            network_packet.append(&mut [128,99].to_vec());
            network_packet.append(&mut seqnum.to_be_bytes().to_vec());
            let _elapsed = &buf[20..22];
            let _elapsed_bytes : [u8;2] = [_elapsed[0] , _elapsed[1]];
            elpased = u16::from_le_bytes(_elapsed_bytes).into();
            elpased *= 90;
            network_packet.append(&mut elpased.to_be_bytes().to_vec());
            network_packet.append(&mut 0_u64.to_be_bytes().to_vec());

            network_packet.append(&mut video_data);
            //vids.send_to(&network_packet, "127.0.0.1:8557").unwrap();
            //vids.send(&network_packet).unwrap();
            seqnum += 1;
//            println!("{}=>{:?}" , network_packet.len() , network_packet.to_vec());            
            video_data = vec![];
            network_packet = vec![];

        }
    }
}



fn start_rtsp(stream: &mut TcpStream) -> bool {
    let mut command: Vec<u8> = vec![171, 205, 0, 8, 0, 0, 1, 255];

    let payload: Vec<u8> = vec![0; 8];

    command.extend_from_slice(&payload);

    let _w = stream.write(&command).unwrap();
    println!("rtsp => wrote {} bytes", _w);

    if _w == 16 {
        return true;
    }
    false
}

/*
fn command(cmd: (u8, u8)) -> Vec<u8> {
    let mut command: Vec<u8> = vec![171, 205, 0, 8, 0, 0, cmd.0, cmd.1];

    let payload: Vec<u8> = vec![0; 8];

    command.extend_from_slice(&payload);

    return command;
}
*/

// Keeps the camera connection alive

fn keep_alive(mut stream: &TcpStream) {
    let mut logged_in = false;
    //let mut got_login_req = false;
    let alive_res: Vec<u8> = vec![171, 205, 0, 0, 0, 0, 1, 19];
    //let alive_req = [171, 205, 0, 0, 0, 0, 1, 18];
    let login_acpt = [171, 205, 0, 129, 0, 0, 1, 17];

    let mut login_res = [0_u8; 8];

    //let rtsp_cmd = [171, 205, 0, 129, 0, 0, 1, 255];

    let sleep_time = time::Duration::from_secs(3);

    loop {
        if !logged_in {
            println!("should print 1 time");
            let _br = stream.read(&mut login_res).unwrap();

            if login_res == login_acpt {
                logged_in = true;

                let mut s2 = stream.try_clone().unwrap();

                let _y = start_rtsp(&mut s2);
                println!(
                    "Login Accepted\n{}",
                    bytes_to_header(&login_res).unwrap().string()
                );
                if _y {
                    //let rtp_server : TcpListener = TcpListener::bind("127.0.0.1:8558").unwrap();
                    thread::spawn(move || rtsp_thread());
                }
            }
        }
        let _bw = stream
            .write(&alive_res)
            .expect("Failed to write alive response");
        if true {}

        println!("FROM ALIVE");
        thread::sleep(sleep_time);
    }
}

fn main() {
    let target = (Ipv4Addr::from_str("192.168.100.1").unwrap(), 6666);

    let login_payload = login_payload();

    let mut con = match TcpStream::connect(target) {
        Ok(u) => u,
        Err(_) => {
            eprintln!("Target Unreachable! {:?}", target);
            exit(1)
        }
    };
    let lbw = con
        .write(&login_payload)
        .expect("Failed to write login request");

    println!("Login Sent! {}", lbw);
    let t = thread::spawn(move || {
        keep_alive(&con);
    });

    t.join().unwrap();
}
