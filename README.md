<p align="center">
  <img src="https://github.com/user-attachments/assets/42d7ec0b-69e5-4731-acc6-e95dbc913f22">
</p>

# hrat

### Alpha version.

### Some of the functions not yet implemented

hrat is a remote access tool (RAT) that offers a wide range of functionalities to remotely control and monitor a target
system. The project is open source and welcomes contributions. The following features are available in hrat:

### Features

### **Software bypass of the LED on webcams less than or equal to 59 hertz will be after the article is released on Medium**

Web Interface

![Interface Preview](https://i.ibb.co/3yf9M28X/image.png)

Polymorphism and Security

- Polymorphism: Bypass antivirus detection using encryption and other polymorphic techniques.

Remote Monitoring

- Remote Screen Viewing: View the target's screen in real-time.
  Webcam Access: Access and view the target's webcam.
  Task Manager Access: Remotely access the task manager.
  Remote CMD: Execute commands on the target system via the command prompt.

Audio

- Text-to-Speech: Convert text to speech on the target system.
  Playing Audio Files: Play recorded audio files on the target system.
  Microphone Listening: Listen to the target's microphone.

Data Retrieval

- Cookies: Retrieve browser cookies.
- Passwords: Retrieve stored passwords.
- Forms: Retrieve form data.
- Credit Card Information: Retrieve stored credit card information.
- Client Data: Retrieve data from various clients such as:
  Steam
  Telegram
  Discord

Keylogger

- Keylogging: Capture and log keystrokes.

Chat

- Chat Application: Communicate with the target through a built-in chat application.

Trolling

- Message Boxes: Display custom message boxes on the target system.
- Opening Links: Open specified links in the target's browser.
- Clipboard Manipulation: Access and modify the clipboard content.
- Mouse Reassignment: Reassign mouse buttons, trigger BSOD, forkbomb, etc.
- Taskbar and Desktop Manipulation: Hide the taskbar, hide desktop icons, flip the screen orientation.

Remote Code Execution

- Code Execution: Execute code remotely in various languages:
  Rust
  C#
  C++
  Batch Scripts
  Powershell

### Usage

1. First you will need to set up webserver.
   Repository contains one, but u can also write your own since it really easy

Repository also contains C&C interface which u can use soon

Install dependencies

```
pip install -r requirements.txt
```

Run Web-client
```
uvicorn main:app --reload --host 0.0.0.0 --port 8000
```

### License

This project is licensed under the MIT License. See the LICENSE file for more details.

Contributions
Contributions are welcome! Please feel free to fork the repository and submit pull requests.
