# Youtube downloader
This is a custom implementation, not the command that everyone use. The original post that inspired me is https://vincentsg.dev/dec31.

### How to use it
```
cargo run "https://www.youtube.com/watch?v=YZhwOWvoR3I"
```

You need to remove pre-existing `video.{whatever_extension}` file from your current folder first.


### How to improve
Here are some possibilities:
 - Song downloader: download only the audio
 - Use adaptative format: can improve the quality
 - Use the user account to get the payload and not the Android trick
