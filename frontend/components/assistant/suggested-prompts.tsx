"use client"

import { ArrowRight } from "lucide-react"
import { Button } from "@/components/ui/button"

interface SuggestedPromptsProps {
  onPromptClick: (prompt: string) => void
}

export function SuggestedPrompts({ onPromptClick }: SuggestedPromptsProps) {
  const prompts = [
    "How do I stake my SOL?",
    "What's the current APY for staking?",
    "How do I unstake my tokens?",
    "What are the risks of staking?",
    "How are rewards calculated?",
  ]

  return (
    <div className="space-y-2">
      <p className="text-sm font-medium">Try asking:</p>
      <div className="flex flex-wrap gap-2">
        {prompts.map((prompt) => (
          <Button
            key={prompt}
            variant="outline"
            size="sm"
            className="flex items-center gap-1"
            onClick={() => onPromptClick(prompt)}
          >
            {prompt}
            <ArrowRight className="h-3 w-3" />
          </Button>
        ))}
      </div>
    </div>
  )
}
