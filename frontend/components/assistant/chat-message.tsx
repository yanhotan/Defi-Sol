import { Bot, User } from "lucide-react"
import { cn } from "@/lib/utils"

type Message = {
  id: string
  role: "user" | "assistant"
  content: string
  timestamp: Date
}

interface ChatMessageProps {
  message: Message
}

export function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === "user"

  return (
    <div
      className={cn(
        "flex w-max max-w-[80%] flex-col gap-2 rounded-lg px-4 py-2 text-sm",
        isUser ? "ml-auto bg-primary text-primary-foreground" : "bg-muted",
      )}
    >
      <div className="flex items-center gap-2">
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
        <span className="font-medium">{isUser ? "You" : "Assistant"}</span>
      </div>
      <div className="whitespace-pre-wrap">{message.content}</div>
      <div className="text-xs opacity-70">
        {message.timestamp.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
      </div>
    </div>
  )
}
