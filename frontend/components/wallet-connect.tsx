"use client"

import { useState } from "react"
import { Wallet } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { useToast } from "@/hooks/use-toast"

export function WalletConnect() {
  const [connected, setConnected] = useState(false)
  const [walletAddress, setWalletAddress] = useState("")
  const { toast } = useToast()

  const handleConnect = () => {
    // Simulate wallet connection
    setTimeout(() => {
      const mockAddress = "GgE5ZbqFfALAYfRHnvDZ1LnCnhiQH6iQMwAiwiALDVT4"
      setWalletAddress(mockAddress)
      setConnected(true)
      toast({
        title: "Wallet connected",
        description: `Connected to ${mockAddress.slice(0, 6)}...${mockAddress.slice(-4)}`,
      })
    }, 1000)
  }

  const handleDisconnect = () => {
    setWalletAddress("")
    setConnected(false)
    toast({
      title: "Wallet disconnected",
      description: "Your wallet has been disconnected",
    })
  }

  if (connected) {
    return (
      <Card>
        <CardContent className="flex items-center justify-between p-4">
          <div className="flex items-center gap-2">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-green-500/20">
              <Wallet className="h-4 w-4 text-green-500" />
            </div>
            <div>
              <p className="text-sm font-medium">Connected Wallet</p>
              <p className="text-xs text-muted-foreground">
                {walletAddress.slice(0, 6)}...{walletAddress.slice(-4)}
              </p>
            </div>
          </div>
          <Button variant="outline" size="sm" onClick={handleDisconnect}>
            Disconnect
          </Button>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardContent className="flex flex-col items-center gap-4 p-6">
        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/20">
          <Wallet className="h-6 w-6 text-primary" />
        </div>
        <div className="text-center">
          <h3 className="text-lg font-medium">Connect Your Wallet</h3>
          <p className="text-sm text-muted-foreground">Connect your Solana wallet to start staking</p>
        </div>
        <Button onClick={handleConnect}>Connect Wallet</Button>
      </CardContent>
    </Card>
  )
}
