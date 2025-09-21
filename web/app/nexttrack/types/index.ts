export interface Track {
  id: number;
  name: string;
}

export interface Preferences {
  mood: number;
  energy: number;
  obscurity: number;
  tempoVariance: number;
  lyricalCoherence: number;
  artistDiversity: boolean;
}

export interface Preset {
  mood: number;
  energy: number;
  obscurity: number;
  tempo: number;
  lyrical: number;
}

// API Types
export interface ApiTrack {
  id: string;
  name: string;
  artist: string;
  features: Record<string, number>;
  popularity: number;
  album_art?: string;
}

export interface RecommendationRequest {
  tracks: string[];
  preferences: {
    energy: number;
    obscurity: number;
    mood: number;
  };
}

// SSE Event Types
export interface StatusEvent {
  type: 'Status';
  message: string;
}

export interface CandidateEvent {
  type: 'Candidate';
  track: ApiTrack;
  score: number;
}

export interface CompleteEvent {
  type: 'Complete';
  tracks: ApiTrack[];
}

export interface ErrorEvent {
  type: 'Error';
  message: string;
}

export interface DebugEvent {
  type: 'Debug';
  message: string;
  data?: any;
}

export type StreamEvent = StatusEvent | CandidateEvent | CompleteEvent | ErrorEvent | DebugEvent;
