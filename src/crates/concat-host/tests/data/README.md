# Audio regression fixture

`speed-ramp-tone.mp3` contains eight seconds of two synthetic sine tones (439 Hz and 997 Hz), generated for this repository, with no recorded music. It exercises MP3 decoder priming and 44.1-to-48 kHz conversion when a speed ramp returns to normal.

Generated with:

```sh
ffmpeg -f lavfi -i 'aevalsrc=0.3*sin(2*PI*439*t)+0.15*sin(2*PI*997*t):s=44100:d=8' -c:a libmp3lame -b:a 64k speed-ramp-tone.mp3
```
