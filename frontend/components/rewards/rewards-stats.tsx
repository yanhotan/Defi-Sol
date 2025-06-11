"use client"

import { useState, useEffect } from "react"
import { Card, CardContent } from "@/components/ui/card"
import { CoinsIcon as CoinsStacked, TrendingUp, Calendar, ArrowUpRight } from "lucide-react"

export function RewardsStats() {
  const [stats, setStats] = useState({
    totalRewards: 0,
    monthlyRewards: 0,
    averageApy: 0,
    nextReward: 0,
  })

  useEffect(() => {
    // Simulate fetching rewards stats
    setStats({
      totalRewards: 0.42,
      monthlyRewards: 0.15,
      averageApy: 6.8,
      nextReward: 12,
    })
  }, [])

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <CoinsStacked className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Total Rewards</p>
            <p className="text-2xl font-bold">{stats.totalRewards} SOL</p>
            <p className="text-xs text-muted-foreground">~${(stats.totalRewards * 142.87).toFixed(2)} USD</p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <Calendar className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Monthly Rewards</p>
            <p className="text-2xl font-bold">{stats.monthlyRewards} SOL</p>
            <p className="text-xs text-muted-foreground">~${(stats.monthlyRewards * 142.87).toFixed(2)} USD</p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <TrendingUp className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Average APY</p>
            <p className="text-2xl font-bold">{stats.averageApy}%</p>
            <p className="text-xs text-green-500">
              <ArrowUpRight className="mr-1 inline h-3 w-3" />
              0.2% from last month
            </p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <CoinsStacked className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Next Reward</p>
            <p className="text-2xl font-bold">~{stats.nextReward} hours</p>
            <p className="text-xs text-muted-foreground">Estimated time until next reward</p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
