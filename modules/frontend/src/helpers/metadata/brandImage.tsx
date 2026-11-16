import { ImageResponse } from "next/og";

type Size = { width: number; height: number };

/** Play-button mark used for generated icons */
export const renderIcon = (size: Size): ImageResponse =>
  new ImageResponse(
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: "#0a0a0a",
        borderRadius: size.width / 5,
      }}
    >
      <svg
        width={size.width / 2}
        height={size.height / 2}
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <path d="M8 5.5v13l10.5-6.5z" fill="#fafafa" />
      </svg>
    </div>,
    size,
  );
