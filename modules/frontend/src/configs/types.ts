import type { AxiosError } from "axios";

export type ApiError = AxiosError & {
  error: string;
};

/**
 * WebSocket close codes (RFC 6455, section 7.4.1).
 * Codes 1005, 1006 and 1015 are reserved and are never sent in a close frame.
 * The browser uses them locally to report why a connection closed.
 */
export enum WSCodes {
  /** The connection did its job and closed cleanly. Do not retry. */
  NormalClosure = 1000,
  /** The server shut down or the client navigated away. Safe to retry. */
  GoingAway = 1001,
  /** Protocol violation. Do not retry, it is a bug. */
  ProtocolError = 1002,
  /** The endpoint received a data type it cannot accept. Do not retry, it is a bug. */
  UnsupportedData = 1003,
  /** Reserved. The close frame had no status code. Retry depends on context. */
  NoStatusReceived = 1005,
  /** Reserved. The connection dropped without a closing handshake. Retry with backoff. */
  AbnormalClosure = 1006,
  /** The payload did not match its type, e.g. non-UTF-8 data in a text frame. Do not retry. */
  InvalidPayloadData = 1007,
  /** Policy violation, often an auth failure. Do not retry; re-authenticate instead. */
  PolicyViolation = 1008,
  /** The message was too large to process. Do not retry; adjust message sizes. */
  MessageTooBig = 1009,
  /** The server did not negotiate the extensions the client required. Do not retry. */
  MandatoryExtension = 1010,
  /** The server hit an unexpected condition. Safe to retry. */
  InternalError = 1011,
  /** The server is restarting. Retry with backoff. */
  ServiceRestart = 1012,
  /** The server is overloaded or rate-limiting. Wait, then retry. */
  TryAgainLater = 1013,
  /** Reserved. The TLS handshake failed. Do not retry. */
  TLSHandshake = 1015,
}
