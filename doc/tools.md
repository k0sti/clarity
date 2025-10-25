
```
FunctionDef {
    name: "transcribe_audio".to_string(),
    description: "Transcribe the contents of an audio file (e.g., mp3) into text".to_string(),
    parameters: json!({
        "type": "object",
        "properties": {
            "file_path": {
                "type": "string",
                "description": "Path to the audio file to transcribe"
            }
        },
        "required": ["file_path"]
    }),
}
```
