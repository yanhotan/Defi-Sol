"use client"

import { useState, useEffect } from "react"
import { Card, CardContent } from "@/components/ui/card"
import { Shield, TrendingUp, Lock, Users } from "lucide-react"

export function PoolStats() {
  const [stats, setStats] = useState({
    basicPool: {
      tvl: 0,
      stakers: 0,
      apy: 0,
    },
    lendingPool: {
      tvl: 0,
      stakers: 0,
      apy: 0,
      utilization: 0,
    },
    lockPool: {
      tvl: 0,
      stakers: 0,
      apy: 0,
      lockedValue: 0,
    },
    totalTVL: 0,
  })

  useEffect(() => {
    // Simulate fetching pool stats
    setStats({
      basicPool: {
        tvl: 1250000,
        stakers: 3240,
        apy: 5.2,
      },
      lendingPool: {
        tvl: 2750000,
        stakers: 1850,
        apy: 7.8,
        utilization: 68,
      },
      lockPool: {
        tvl: 4500000,
        stakers: 1120,
        apy: 12.5,
        lockedValue: 3800000,
      },
      totalTVL: 8500000,
    })
  }, [])

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <Shield className="h-4 w-4 text-green-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Basic Pool TVL</p>
            <p className="text-2xl font-bold">{(stats.basicPool.tvl / 1000000).toFixed(1)}M SOL</p>
            <p className="text-xs text-muted-foreground">
              {stats.basicPool.stakers.toLocaleString()} stakers • {stats.basicPool.apy}% APY
            </p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <TrendingUp className="h-4 w-4 text-yellow-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Lending Pool TVL</p>
            <p className="text-2xl font-bold">{(stats.lendingPool.tvl / 1000000).toFixed(1)}M SOL</p>
            <p className="text-xs text-muted-foreground">
              {stats.lendingPool.stakers.toLocaleString()} stakers • {stats.lendingPool.apy}% APY
            </p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <Lock className="h-4 w-4 text-red-500" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Lock Pool TVL</p>
            <p className="text-2xl font-bold">{(stats.lockPool.tvl / 1000000).toFixed(1)}M SOL</p>
            <p className="text-xs text-muted-foreground">
              {stats.lockPool.stakers.toLocaleString()} stakers • {stats.lockPool.apy}% APY
            </p>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardContent className="flex flex-col gap-2 p-6">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/20">
            <Users className="h-4 w-4 text-primary" />
          </div>
          <div className="space-y-1">
            <p className="text-sm font-medium text-muted-foreground">Total Value Locked</p>
            <p className="text-2xl font-bold">${(stats.totalTVL / 1000000).toFixed(1)}M</p>
            <p className="text-xs text-muted-foreground">
              {(stats.basicPool.stakers + stats.lendingPool.stakers + stats.lockPool.stakers).toLocaleString()} total
              stakers
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
