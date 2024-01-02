use serde::Deserialize;
use serde_json::json;
use url::Url;
use std::{collections::HashMap, process};
use std::io::Write;
use futures_util::StreamExt;

#[derive(Debug, Deserialize, Clone)]
struct Format {
    url: String,
    #[serde(rename = "qualityLabel")]
    quality_label: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

#[derive(Debug, Deserialize)]
struct StreamingData {
    formats: Vec<Format>,
}

#[derive(Debug, Deserialize)]
struct PlayerResponse {
    #[serde(rename = "streamingData")]
    streaming_data: StreamingData,
}


#[tokio::main]
async fn main() {
    let args : Vec<String> = std::env::args().collect();
    let url = match args.get(1) {
        Some(v) => v,
        None => {
            eprintln!("Please give a video url.");
            process::exit(1);
        }
    };

    let parsed_url = match Url::parse(&url) {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("Please give a valide url.");
            process::exit(1);
        }
    };

    match parsed_url.domain() {
        Some("www.youtube.com") => (),
        Some("youtube.com") => (),
        _ => {
            eprintln!("You must give a Youtube url.");
            process::exit(1);
        }
    }

    let formats = get_video_format(parsed_url).await;
    let best_format = get_best_format(&formats);

    download_video(best_format).await;
}

async fn get_video_format(parsed_url: Url) -> Vec<Format> {
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

    let video_id = match hash_query.get(&String::from('v')) {
        Some(value) => value,
        None => {
            "You should register a valid url";
            process::exit(1);
        }
    };

    let data = json!({
        "videoId": video_id,
        "context": {
            "client": {
                "hl": "en",
                "gl": "US",
                "clientName": "ANDROID",
                "clientVersion": "17.10.35",
                "androidSdkVersion": 30,
                "osName": "Android",
                "osVersion": "12",
                "platform": "MOBILE"
            }
        },
        "params": "2AMBCgIQBg",
        "contentCheckOk": true,
        "racyCheckOk": true,
        "user": {
          "lockedSafetyMode": false
        },
        "playbackContext": {
          "contentPlaybackContext": {
            "html5Preference": "HTML5_PREF_WANTS",
          },
        },
    });

    let client = reqwest::Client::new();
    let resp: PlayerResponse = client.post("https://www.youtube.com/youtubei/v1/player?key=AIzaSyA8eiZmM1FaDVjRy-df2KTyQ_vz_yYM39w")
        .body(data.to_string())
        .send().await.unwrap()
        .json().await.unwrap();

    resp.streaming_data.formats
}

// There is also a jsonResult.streamingData.adaptiveFormats which can contains better format (for example 1080p)
// but in constrast with "formats" that contains both video and audio, "adaptativeFormats" are only one of those two. This mean we
// have to download both and merge them together.
fn get_best_format(formats: &Vec<Format>) -> &Format {
    let best_qualities = ["1080p", "720p", "360p", "144p"];
    let best_encodings = ["video/mp4", "video/webm"];

    let mut matching_format: Option<&Format> = None;

    'outer: for wanted_quality in best_qualities {
        for wanted_encoding in best_encodings {
            matching_format = formats.into_iter().find(|f| f.quality_label == wanted_quality && f.mime_type.contains(wanted_encoding));
            if matching_format.is_some() {
                break 'outer;
            }
        }
    }

    match matching_format {
        Some(v) => v,
        None => {
            eprintln!("No good format found for this video.");
            process::exit(1);
        }
    }
}

async fn download_video(format: &Format) {
    let encoding = format.mime_type.split(";").collect::<Vec<&str>>()[0];
    let file_extension = encoding.split("/").collect::<Vec<&str>>()[1];

    let current_dir = std::env::current_dir().unwrap();
    let fname = current_dir.as_path().join("video.".to_string() + file_extension);

    let mut video_file = std::fs::File::create(&fname).unwrap();

    let res = reqwest::get(&format.url).await.unwrap();
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading the file"))).unwrap();
        video_file.write(&chunk).or(Err(format!("Error while writing to file"))).unwrap();
    }

    println!("Video available at {}", fname.as_path().display());
}