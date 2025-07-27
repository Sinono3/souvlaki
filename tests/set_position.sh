#!/bin/bash
dbus-send --dest=org.mpris.MediaPlayer2.souvlaki_player --print-reply /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player.SetPosition objpath:"/" int64:5000000

