"use client"

import { useState, useEffect } from "react"
import { ExternalLink } from "lucide-react"
import { Button } from "@/components/ui/button"

export function DashboardHeader() {
  const [solPrice, setSolPrice] = useState<number | null>(null)
  const [priceChange, setPriceChange] = useState<number | null>(null)

  useEffect(() => {
    // Simulate fetching SOL price data
    setSolPrice(142.87)
    setPriceChange(3.24)
  }, [])

  return (
    <div className="flex flex-col gap-2">
      <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between">
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        {solPrice && (
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium">SOL: ${solPrice.toFixed(2)}</span>
            <span className={`text-xs ${priceChange && priceChange >= 0 ? "text-green-500" : "text-red-500"}`}>
              {priceChange && priceChange >= 0 ? "+" : ""}
              {priceChange?.toFixed(2)}%
            </span>
            <Button variant="ghost" size="icon" className="h-6 w-6">
              <ExternalLink className="h-4 w-4" />
            </Button>
          </div>
        )}
      </div>
      <p className="text-muted-foreground">Welcome to your Solana staking dashboard</p>
    </div>
  )
}
