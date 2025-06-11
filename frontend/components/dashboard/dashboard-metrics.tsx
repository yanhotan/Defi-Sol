"use client"

import { useState, useEffect } from "react"
import { ArrowUpRight, CoinsIcon as CoinsStacked, Percent, Timer } from "lucide-react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

export function DashboardMetrics() {
  const [metrics, setMetrics] = useState({
    totalStaked: 0,
    apy: 0,
    rewards: 0,
    nextReward: 0,
  })

  useEffect(() => {
    // Simulate fetching metrics data
    setMetrics({
      totalStaked: 24.5,
      apy: 6.8,
      rewards: 0.42,
      nextReward: 12,
    })
  }, [])

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Total Staked</CardTitle>
          <CoinsStacked className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.totalStaked} SOL</div>
          <p className="text-xs text-muted-foreground">~${(metrics.totalStaked * 142.87).toFixed(2)} USD</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Current APY</CardTitle>
          <Percent className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.apy}%</div>
          <p className="text-xs text-muted-foreground">
            <span className="text-green-500">
              <ArrowUpRight className="mr-1 inline h-3 w-3" />
              0.2%
            </span>{" "}
            from last week
          </p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Total Rewards</CardTitle>
          <CoinsStacked className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.rewards} SOL</div>
          <p className="text-xs text-muted-foreground">~${(metrics.rewards * 142.87).toFixed(2)} USD</p>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Next Reward</CardTitle>
          <Timer className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.nextReward} hours</div>
          <p className="text-xs text-muted-foreground">Estimated time until next reward</p>
        </CardContent>
      </Card>
    </div>
  )
}
