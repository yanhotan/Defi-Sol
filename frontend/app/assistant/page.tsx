import { AssistantChat } from "@/components/assistant/assistant-chat"
import { WalletConnect } from "@/components/wallet-connect"

export default function AssistantPage() {
  return (
    <div className="flex flex-col gap-6 p-6 md:p-8">
      <div className="flex flex-col gap-2">
        <h1 className="text-3xl font-bold tracking-tight">Staking Assistant</h1>
        <p className="text-muted-foreground">Chat with our AI assistant for help with staking, rewards, and more</p>
      </div>
      <WalletConnect />
      <AssistantChat />
    </div>
  )
}
