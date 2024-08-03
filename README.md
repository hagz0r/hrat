# hrat

### Alpha version.
### Some of the functions not yet implemented

hrat is a remote access tool (RAT) that offers a wide range of functionalities to remotely control and monitor a target
system. The project is open source and welcomes contributions. The following features are available in hrat:

### Features

Polymorphism and Security

+ Polymorphism: Bypass antivirus detection using encryption and other polymorphic techniques.

Remote Monitoring

+ Remote Screen Viewing: View the target's screen in real-time.
  Webcam Access: Access and view the target's webcam.
  Task Manager Access: Remotely access the task manager.
  Remote CMD: Execute commands on the target system via the command prompt.

Audio

+ Text-to-Speech: Convert text to speech on the target system.
  Playing Audio Files: Play recorded audio files on the target system.
  Microphone Listening: Listen to the target's microphone.

Data Retrieval

+ Cookies: Retrieve browser cookies.
+ Passwords: Retrieve stored passwords.
+ Forms: Retrieve form data.
+ Credit Card Information: Retrieve stored credit card information.
+ Client Data: Retrieve data from various clients such as:
  Steam
  Telegram
  Discord

Keylogger

+ Keylogging: Capture and log keystrokes.

Chat

+ Chat Application: Communicate with the target through a built-in chat application.

Trolling

+ Message Boxes: Display custom message boxes on the target system.
+ Opening Links: Open specified links in the target's browser.
+ Clipboard Manipulation: Access and modify the clipboard content.
+ Mouse Reassignment: Reassign mouse buttons, trigger BSOD, forkbomb, etc.
+ Taskbar and Desktop Manipulation: Hide the taskbar, hide desktop icons, flip the screen orientation.

Remote Code Execution

+ Code Execution: Execute code remotely in various languages:
  Rust
  C#
  C++
  Batch Scripts
  Powershell

### Usage

1. First you will need to set up webserver. Repository contains one, but u can also write your own since it really easy
2. Build client. To build one you need to provide server ip-address and port, default one is 4040

``` 
python3 server.py
cargo b <ip> <port>
```

### License

This project is licensed under the MIT License. See the LICENSE file for more details.

Contributions
Contributions are welcome! Please feel free to fork the repository and submit pull requests.