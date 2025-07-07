declare module 'react-speech-kit' {
  export interface UseSpeechRecognitionOptions {
    onResult: (result: string) => void;
    onError?: (error: Error | unknown) => void;
  }

  export interface UseSpeechRecognitionReturn {
    listen: () => void;
    listening: boolean;
    stop: () => void;
  }

  export function useSpeechRecognition(
    options: UseSpeechRecognitionOptions
  ): UseSpeechRecognitionReturn;
} 