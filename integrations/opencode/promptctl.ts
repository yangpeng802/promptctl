// promptctl OpenCode integration
import type { Plugin } from "@opencode-ai/plugin"

const COMMAND = "pm-fix"

export const PromptctlPlugin: Plugin = async ({ $, directory }) => ({
  config: async (config) => {
    config.command ??= {}
    config.command[COMMAND] = {
      description: "Generate a FIX prompt with promptctl",
      template: "$ARGUMENTS",
    }
  },

  "command.execute.before": async (input, output) => {
    if (input.command !== COMMAND) return

    const task = input.arguments.trim()
    if (!task) {
      throw new Error("Usage: /pm-fix <task>")
    }

    // Bun shell escapes interpolated values, so the complete task is passed as
    // one argv item instead of being interpreted as shell syntax.
    const prompt = (await $`pm fix ${task} --no-copy`.cwd(directory).text()).trim()
    if (!prompt) {
      throw new Error("promptctl returned an empty prompt")
    }

    const textPart = output.parts.find((part) => part.type === "text")
    if (!textPart) {
      throw new Error("OpenCode did not provide a text part for /pm-fix")
    }
    textPart.text = prompt
  },
})
