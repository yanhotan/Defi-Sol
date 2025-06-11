"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import { Shield, TrendingUp, Lock } from "lucide-react"

export function PoolsOverview() {
  const [poolData, setPoolData] = useState({
    totalTVL: 8500000,
    distribution: {
      basic: 1250000,
      lending: 2750000,
      lock: 4500000,
    },
    lockPeriods: {
      "30days": 1200000,
      "90days": 1500000,
      "180days": 1000000,
      "270days": 500000,
      "365days": 300000,
    },
  })

  useEffect(() => {
    // Simulate fetching pool data
    // In a real app, this would be an API call
  }, [])

  // Calculate percentages
  const basicPercent = (poolData.distribution.basic / poolData.totalTVL) * 100
  const lendingPercent = (poolData.distribution.lending / poolData.totalTVL) * 100
  const lockPercent = (poolData.distribution.lock / poolData.totalTVL) * 100

  // Calculate lock period percentages
  const thirtyDaysPercent = (poolData.lockPeriods["30days"] / poolData.distribution.lock) * 100
  const ninetyDaysPercent = (poolData.lockPeriods["90days"] / poolData.distribution.lock) * 100
  const oneEightyDaysPercent = (poolData.lockPeriods["180days"] / poolData.distribution.lock) * 100
  const twoSeventyDaysPercent = (poolData.lockPeriods["270days"] / poolData.distribution.lock) * 100
  const yearPercent = (poolData.lockPeriods["365days"] / poolData.distribution.lock) * 100

  return (
    <Card>
      <CardHeader>
        <CardTitle>Pools Overview</CardTitle>
        <CardDescription>Distribution of staked SOL across pools</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <h3 className="text-sm font-medium">Pool Distribution</h3>
          <div className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Shield className="h-4 w-4 text-green-500" />
                  <span className="text-sm">Basic Pool</span>
                </div>
                <span className="text-sm font-medium">{basicPercent.toFixed(1)}%</span>
              </div>
              <Progress value={basicPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.distribution.basic / 1000000).toFixed(1)}M SOL
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <TrendingUp className="h-4 w-4 text-yellow-500" />
                  <span className="text-sm">Lending Pool</span>
                </div>
                <span className="text-sm font-medium">{lendingPercent.toFixed(1)}%</span>
              </div>
              <Progress value={lendingPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.distribution.lending / 1000000).toFixed(1)}M SOL
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <Lock className="h-4 w-4 text-red-500" />
                  <span className="text-sm">Lock Pool</span>
                </div>
                <span className="text-sm font-medium">{lockPercent.toFixed(1)}%</span>
              </div>
              <Progress value={lockPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.distribution.lock / 1000000).toFixed(1)}M SOL
              </p>
            </div>
          </div>
        </div>

        <div className="space-y-4">
          <h3 className="text-sm font-medium">Lock Pool Periods</h3>
          <div className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm">1 Month</span>
                <span className="text-sm font-medium">{thirtyDaysPercent.toFixed(1)}%</span>
              </div>
              <Progress value={thirtyDaysPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.lockPeriods["30days"] / 1000000).toFixed(1)}M SOL
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm">3 Months</span>
                <span className="text-sm font-medium">{ninetyDaysPercent.toFixed(1)}%</span>
              </div>
              <Progress value={ninetyDaysPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.lockPeriods["90days"] / 1000000).toFixed(1)}M SOL
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm">6 Months</span>
                <span className="text-sm font-medium">{oneEightyDaysPercent.toFixed(1)}%</span>
              </div>
              <Progress value={oneEightyDaysPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.lockPeriods["180days"] / 1000000).toFixed(1)}M SOL
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm">9 Months</span>
                <span className="text-sm font-medium">{twoSeventyDaysPercent.toFixed(1)}%</span>
              </div>
              <Progress value={twoSeventyDaysPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.lockPeriods["270days"] / 1000000).toFixed(1)}M SOL
              </p>
            </div>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <span className="text-sm">1 Year</span>
                <span className="text-sm font-medium">{yearPercent.toFixed(1)}%</span>
              </div>
              <Progress value={yearPercent} className="h-2" />
              <p className="text-xs text-muted-foreground text-right">
                {(poolData.lockPeriods["365days"] / 1000000).toFixed(1)}M SOL
              </p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
