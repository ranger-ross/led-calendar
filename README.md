This is a small program that syncs my Google Calendar to the [badgemagic](https://badgemagic.fossasia.org/) led board I got when I went to Rust Asia 2025

This program fetches calendar events from the Google Calendar API, formats a message for the LED board, and uploads them over USB.
It also coverts Japanese to English as the embedded graphic library does not appear to support Kanji/Kana.
