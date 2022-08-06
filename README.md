## A program to connect to my SJcam 4000Air actioncamera and capture live feed

**It is very early stage prototype**

### Information
This program connects to my sjcam 4000 air actioncamera, fetches live stream (raw headless h264 data) and dumps it in the current working directory. 

I got huge help from this blog post [Hacking the TecTecTec! XPro2 Action Camera](https://blog.jonaskoeritz.de/2017/02/21/hacking-the-xpro2-action-camera/) by Jonas Köritz, and his github project [actioncam](https://github.com/jonas-koeritz/actioncam).

I chose to use Rust, because I was more comfortable with it than Go.

It could've implemented a RTSP feed, but my goal for this project was to record the livestream from my computer without pushing a SD card into the camera. Well, it kinda works, but I am doing little research to how convert the raw h264 data to any universal media format such as MKV or MP4 or WAV. It's gonna take some time for me to figure out this.

### Usage

* `cargo run .` (start the reciever server) (starts saving the stream as `xcal.h264`)
* `./playvlc` (play the live stream with vlc [stream has about 4-5 seconds of delay] ) (You need to have VLC media player installed)
* `./convert2mp4` (convert the saved stream to `abcd.mp4` file) (**Please Stop the rust application before converting**, as the script will never finish the job) (You need to have ffmpeg installed with h264 codecs)




### Contribute
If you any improvement ideas or any solution to any bugs, feel to free to raise an issue or send a patch. If you know how to convert this raw live data to any media format, then please help.
