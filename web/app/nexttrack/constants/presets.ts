import type { Preset } from "../types";

export const presets: Record<string, Preset> = {
  discovery: {
    mood: 0.5,
    energy: 0.6,
    obscurity: 0.8,
    tempo: 0.4,
    lyrical: 0.3,
  },
  chill: { mood: 0.3, energy: 0.2, obscurity: 0.5, tempo: 0.2, lyrical: 0.7 },
  energy: { mood: 0.8, energy: 0.9, obscurity: 0.4, tempo: 0.7, lyrical: 0.4 },
  hits: { mood: 0.7, energy: 0.7, obscurity: 0.2, tempo: 0.5, lyrical: 0.5 },
};
