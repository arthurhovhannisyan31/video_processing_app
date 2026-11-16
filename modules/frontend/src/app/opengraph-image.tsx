import { SITE_NAME } from "configs/constants";
import { ImageResponse } from "next/og";

export const alt = `${SITE_NAME} – compress & convert videos online`;
export const size = { width: 1200, height: 630 };
export const contentType = "image/png";

export default function OpengraphImage() {
  return new ImageResponse(
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        justifyContent: "center",
        padding: 96,
        gap: 24,
        background: "#0a0a0a",
        color: "#fafafa",
      }}
    >
      <div style={{ fontSize: 40, opacity: 0.7 }}>{SITE_NAME}</div>
      <div style={{ fontSize: 80, fontWeight: 700, lineHeight: 1.1 }}>
        Compress &amp; convert videos online
      </div>
      <div style={{ fontSize: 36, opacity: 0.7 }}>
        MP4 · MKV · WebM · MOV — H.264 &amp; VP9
      </div>
    </div>,
    size,
  );
}
