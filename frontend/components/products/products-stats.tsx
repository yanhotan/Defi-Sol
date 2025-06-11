"use client"

import { useState, useEffect } from "react"
import { Card, CardContent } from "@/components/ui/card"
import { CoinsIcon, DollarSign, Layers, Users } from "lucide-react"

export function ProductsStats() {
  const [stats, setStats] = useState({
    msolTVL: 4500000,
    usdcTVL: 2750000,
    msolUsdcTVL: 8500000,
    totalUsers: 6210,
  })

  useEffect(() => {
    // Simulate fetching stats data
  }, [])

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <CoinsIcon className="h-4 w-4 text-blue-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">mSOL TVL</p>
            <p className="text-2xl font-bold">{(stats.msolTVL / 1000000).toFixed(1)}M SOL</p>
            <p className="text-xs text-muted-foreground">~${(stats.msolTVL * 0.01).toLocaleString()} USD</p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <DollarSign className="h-4 w-4 text-green-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">USDC TVL</p>
            <p className="text-2xl font-bold">${(stats.usdcTVL / 1000000).toFixed(1)}M</p>
            <p className="text-xs text-muted-foreground">Stable yield product</p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <Layers className="h-4 w-4 text-purple-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">mSOL-USDC TVL</p>
            <p className="text-2xl font-bold">${(stats.msolUsdcTVL / 1000000).toFixed(1)}M</p>
            <p className="text-xs text-muted-foreground">Dual asset product</p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <Users className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Total Users</p>
            <p className="text-2xl font-bold">{stats.totalUsers.toLocaleString()}</p>
            <p className="text-xs text-muted-foreground">Across all products</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
