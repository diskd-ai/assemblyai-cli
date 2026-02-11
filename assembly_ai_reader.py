import os
import logging
from pathlib import Path
from typing import Dict, List, Optional, Union
from enum import Enum
from tempfile import NamedTemporaryFile
from llama_index.readers.file import FlatReader
import assemblyai as aai
from dotenv import load_dotenv
from llama_index.core.readers.base import BaseReader
from llama_index.core.schema import Document
from fsspec import AbstractFileSystem

# Import pydub only when needed to avoid import errors if not installed

logger = logging.getLogger(__name__)

class SpeechModel(str, Enum):
    """Speech model options for AssemblyAI."""
    BEST = "best"
    NANO = "nano"

class TranscriptFormat(str, Enum):
    """Format options for transcript output."""
    TEXT = "text"
    SRT = "srt"
    VTT = "vtt"
    
aai.settings.base_url = "https://api.eu.assemblyai.com"

class AssemblyAITranscriptReader(BaseReader):
    """Audio/Video transcription reader using AssemblyAI API.
    
    Extract text from transcript of video/audio files using AssemblyAI's speech-to-text service.
    """

    def __init__(
        self, 
        api_key: Optional[str] = None,
        speech_model: SpeechModel = SpeechModel.BEST,
        language_detection: bool = True,
        language: Optional[str] = None,
        punctuate: bool = True,
        format_text: bool = True,
        disfluencies: bool = False,
        filter_profanity: bool = False,
        word_boost: Optional[List[str]] = None,
        custom_spelling: Optional[List[Dict[str, str]]] = None,
        output_format: TranscriptFormat = TranscriptFormat.TEXT,
        chars_per_caption: int = 128,
        speech_threshold: Optional[float] = None,        
        speaker_labels: bool = False,
        multichannel: bool = True,
        *args, 
        **kwargs
    ) -> None:
        """Initialize the reader.
        
        Args:
            api_key: AssemblyAI API key. If not provided, will try to get from environment variable.
            speech_model: The model to use for transcription ('best' or 'nano')
            language_detection: Whether to automatically detect the language
            language: The language code if known (disables auto-detection)
            punctuate: Whether to include punctuation in the transcript
            format_text: Whether to format text (capitalization, number formatting)
            disfluencies: Whether to include filler words (um, uh, etc.)
            filter_profanity: Whether to filter out profanity
            word_boost: List of words or phrases to boost recognition
            custom_spelling: List of custom spelling replacements
            output_format: The format of the transcript output (text, srt, vtt)
            chars_per_caption: Number of characters per caption for srt/vtt formats
            speech_threshold: Minimum percentage of speech required (0.0-1.0)
        """
        super().__init__(*args, **kwargs)
        self.api_key = api_key or os.environ.get("ASSEMBLYAI_API_KEY")
        if not self.api_key:
            raise ValueError(
                "AssemblyAI API key must be provided as an argument or "
                "set as environment variable 'ASSEMBLYAI_API_KEY'"
            )
        
        # Set API key for AssemblyAI
        aai.settings.api_key = self.api_key
        
        # Store configuration parameters
        self.speech_model = getattr(aai.SpeechModel, speech_model.value)
        self.language_detection = language_detection
        self.language = language
        self.punctuate = punctuate
        self.format_text = format_text
        self.disfluencies = disfluencies
        self.filter_profanity = filter_profanity
        self.word_boost = word_boost
        self.custom_spelling = custom_spelling
        self.output_format = output_format
        self.chars_per_caption = chars_per_caption
        self.speech_threshold = speech_threshold
        self.speaker_labels = speaker_labels
        self.multichannel = multichannel
    
    def load_data(
        self,
        file: Union[str, Path],
        extra_info: Optional[Dict] = None,
        fs: Optional[AbstractFileSystem] = None,
    ) -> List[Document]:
        """Process audio/video file and return document with transcription.
        
        Args:
            file: Path to the audio/video file or URL
            extra_info: Optional metadata to add to the document
            fs: Optional filesystem to use
            
        Returns:
            List containing a single Document with the transcription
        """
        try:
            # Check if file is a URL
            is_url = isinstance(file, str) and file.startswith(('http://', 'https://'))
            
            if is_url:
                file_path = file
            else:
                # Convert file to Path object if it's a string
                if isinstance(file, str):
                    file = Path(file)
                    
                # Handle video files by extracting audio if needed
                file_path = self._prepare_audio_file(file, fs)
            
            # Transcribe with AssemblyAI API
            transcript = self._transcribe_with_assemblyai(str(file_path))
            
            # Clean up temporary file if one was created
            if not is_url and file_path != str(file):
                try:
                    os.remove(file_path)
                except Exception as e:
                    logger.warning(f"Failed to remove temporary file {file_path}: {e}")
                
            documents = []
            
            with NamedTemporaryFile(delete=True) as temp_file:
                temp_file.write(transcript.encode('utf-8'))
                temp_file.flush()
                documents = FlatReader().load_data(file=Path(temp_file.name))
                
            return documents
            
        except Exception as e:
            logger.error(f"Error transcribing file {file}: {str(e)}")
            raise

    def _prepare_audio_file(self, file: Path, fs: Optional[AbstractFileSystem] = None) -> str:
        """Prepare audio file for transcription, converting video to audio if needed.
        
        Args:
            file: Path to the audio/video file
            fs: Optional filesystem to use
            
        Returns:
            Path to the prepared audio file
        """
        file_path = str(file)
        
        # If it's already an audio file, return the path
        if file_path.lower().endswith(('.mp3', '.wav', '.flac', '.m4a', '.ogg')):
            return file_path
        
        # If it's a video format, extract audio
        if file_path.lower().endswith(('.mp4', '.avi', '.mov', '.mkv', '.webm')):
            try:
                # Import pydub only when needed
                from pydub import AudioSegment # type: ignore
                
                # Create a temporary file for the audio
                temp_audio = NamedTemporaryFile(suffix='.mp3', delete=False)
                temp_audio_path = temp_audio.name
                temp_audio.close()
                
                if fs:
                    with fs.open(file, "rb") as f:
                        video = AudioSegment.from_file(f)
                else:
                    video = AudioSegment.from_file(file_path)

                # Extract audio and save to temp file
                audio = video.split_to_mono()[0]
                audio.export(temp_audio_path, format="mp3")
                
                return temp_audio_path
            except ImportError:
                raise ImportError("Please install pydub: 'pip install pydub'")
            except Exception as e:
                logger.error(f"Error extracting audio: {str(e)}")
                raise
        
        return file_path

    def _transcribe_with_assemblyai(self, file_path: str) -> str:
        """Transcribe audio file using AssemblyAI API.
        
        Args:
            file_path: Path to the audio file or URL
            
        Returns:
            Transcribed text
        """
        
        # override the `Transcriber`'s config with a new config
        # transcriber.config = aai.TranscriptionConfig(punctuate=False, format_text=False)

        config_params = {
            "speech_model": self.speech_model,
            "language_detection": self.language_detection,
            "punctuate": self.punctuate,
            "format_text": self.format_text,
            "disfluencies": self.disfluencies,
            "filter_profanity": self.filter_profanity,
            "speech_threshold": self.speech_threshold,
            "punctuate": self.punctuate,
            "format_text": self.format_text,
            "speaker_labels": self.speaker_labels,
            "multichannel": self.multichannel
        }
        
        # Add optional parameters only if they are not None
        if self.word_boost:
            config_params["word_boost"] = self.word_boost
        
        if self.custom_spelling:
            config_params["custom_spelling"] = self.custom_spelling
            
        if self.language and not self.language_detection:
            config_params["language"] = self.language
            
        # Create config and transcriber
        config = aai.TranscriptionConfig(**config_params)
        transcriber = aai.Transcriber(config=config)
        
        # Transcribe the file
        transcript = transcriber.transcribe(file_path)
        
        # Check for errors
        if transcript.status == aai.TranscriptStatus.error:
            error_msg = f"Transcription failed: {transcript.error}"
            logger.error(error_msg)
            raise Exception(error_msg)
        
        # Return the transcript based on requested format
        if self.output_format == TranscriptFormat.SRT:
            return transcript.export_subtitles_srt(chars_per_caption=self.chars_per_caption)
        elif self.output_format == TranscriptFormat.VTT:
            return transcript.export_subtitles_vtt(chars_per_caption=self.chars_per_caption)
        else:
            return transcript.text or ""


if __name__ == "__main__":
    load_dotenv("../.env")
    # file_path = "/Users/alexeus/iosya/files/AUDIO-2025-03-19-14-21-57.m4a"
    file_path = "/Users/alexeus/iosya/files/univer_admin.ogg"    
    reader = AssemblyAITranscriptReader(
        format_text=True,
        punctuate=True,
        speaker_labels=True,  
        multichannel=False,
        output_format=TranscriptFormat.SRT,
        chars_per_caption=256
    ).load_data(file=file_path) 
    
    # replace the file extension with .txt
    target_file = os.path.splitext(file_path)[0] + ".txt"
    print(f"Writing transcript to: {target_file}")
    with open(target_file, "w") as f:
        f.write("\n".join([doc.get_content() for doc in reader]))
        
        