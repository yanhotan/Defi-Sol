"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Copy, ExternalLink } from "lucide-react"
import { useToast } from "@/hooks/use-toast"

export function AccountDetails() {
  const [accountData, setAccountData] = useState({
    address: "",
    balance: 0,
    stakedBalance: 0,
    rewards: 0,
  })
  const { toast } = useToast()

  useEffect(() => {
    // Simulate fetching account data
    setAccountData({
      address: "GgE5ZbqFfALAYfRHnvDZ1LnCnhiQH6iQMwAiwiALDVT4",
      balance: 30,
      stakedBalance: 24.5,
      rewards: 0.42,
    })
  }, [])

  const handleCopyAddress = () => {
    navigator.clipboard.writeText(accountData.address)
    toast({
      title: "Address copied",
      description: "Wallet address copied to clipboard",
    })
  }

  const formatAddress = (address: string) => {
    if (!address) return ""
    return `${address.slice(0, 6)}...${address.slice(-4)}`
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Account Details</CardTitle>
        <CardDescription>Your Solana wallet and staking information</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
          <p className="text-sm text-muted-foreground">Wallet Address</p>
          <div className="flex items-center gap-2">
            <code className="rounded bg-muted px-2 py-1 text-sm">{formatAddress(accountData.address)}</code>
            <Button variant="ghost" size="icon" onClick={handleCopyAddress}>
              <Copy className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" asChild>
              <a
                href={`https://explorer.solana.com/address/${accountData.address}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                <ExternalLink className="h-4 w-4" />
              </a>
            </Button>
          </div>
        </div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Available Balance</p>
            <p className="text-lg font-medium">{accountData.balance} SOL</p>
            <p className="text-xs text-muted-foreground">~${(accountData.balance * 142.87).toFixed(2)} USD</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Staked Balance</p>
            <p className="text-lg font-medium">{accountData.stakedBalance} SOL</p>
            <p className="text-xs text-muted-foreground">~${(accountData.stakedBalance * 142.87).toFixed(2)} USD</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Total Rewards</p>
            <p className="text-lg font-medium">{accountData.rewards} SOL</p>
            <p className="text-xs text-muted-foreground">~${(accountData.rewards * 142.87).toFixed(2)} USD</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Staking APY</p>
            <p className="text-lg font-medium">6.8%</p>
            <p className="text-xs text-muted-foreground">Current annual percentage yield</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
