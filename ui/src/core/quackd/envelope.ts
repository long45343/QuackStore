import { z } from "zod"

/** Envelope version this WebUI understands (`duck_core::envelope::API_VERSION`). */
export const ENVELOPE_API = 1

const errorSchema = z.object({
  code: z.string(),
  message: z.string(),
  details: z.unknown().optional(),
})

const envelopeSchema = z.object({
  ok: z.boolean(),
  api: z.number().optional(),
  command: z.string(),
  data: z.unknown(),
  error: errorSchema.nullable().optional(),
  ts: z.number().optional(),
})

export type Envelope = z.infer<typeof envelopeSchema>

/** A failed backend command, identified by the stable `code` the backend reports. */
export class QuackError extends Error {
  readonly code: string
  readonly command: string
  readonly details: unknown

  constructor(code: string, message: string, command: string, details?: unknown) {
    super(message)
    this.name = "QuackError"
    this.code = code
    this.command = command
    this.details = details
  }
}

export function isQuackError(error: unknown): error is QuackError {
  return error instanceof QuackError
}

/**
 * Returns the last line of stdout that is a valid envelope. The shell wrapper and the root
 * manager may print warnings before it.
 */
export function parseEnvelope(stdout: string, stderr: string, command: string): Envelope {
  const lines = stdout.split(/\r?\n/).reverse()
  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed.startsWith("{")) continue
    try {
      const parsed = envelopeSchema.safeParse(JSON.parse(trimmed))
      if (parsed.success) return parsed.data
    } catch {
      // Not JSON; keep looking.
    }
  }
  throw new QuackError(
    "no_envelope",
    stderr.trim() || stdout.trim() || "the backend printed no response",
    command,
    { stdout, stderr },
  )
}
