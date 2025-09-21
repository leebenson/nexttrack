import type { Route } from "./+types/home";
import NextTrack from "../nexttrack/nexttrack";

export function meta({}: Route.MetaArgs) {
  return [
    { title: "NextTrack - Privacy-First Music Recommendations" },
    {
      name: "description",
      content:
        "Discover your next favorite track with NextTrack's privacy-first music recommendation engine.",
    },
  ];
}

export default function Home() {
  return <NextTrack />;
}
