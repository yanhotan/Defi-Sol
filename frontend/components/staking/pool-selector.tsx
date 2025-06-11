"use client"

import type React from "react"

import { useState } from "react"
import { Shield, TrendingUp, Lock } from "lucide-react"
import { Card, CardContent } from "@/components/ui/card"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import { useToast } from "@/hooks/use-toast"
import { cn } from "@/lib/utils"

export type PoolType = "basic" | "lending" | "lock"

interface PoolInfo {
  type: PoolType
  name: string
  description: string
  riskLevel: "Low" | "Medium" | "High"
  apy: number
  liquidityStatus: string
  icon: React.ElementType
  color: string
}

export const poolData: Record<PoolType, PoolInfo> = {
  basic: {
    type: "basic",
    name: "Basic Pool",
    description: "Low-risk staking with instant unstake capability",
    riskLevel: "Low",
    apy: 5.2,
    liquidityStatus: "Instant unstake",
    icon: Shield,
    color: "text-green-500",
  },
  lending: {
    type: "lending",
    name: "Lending Pool",
    description: "Medium-risk staking with interest from borrowers",
    riskLevel: "Medium",
    apy: 7.8,
    liquidityStatus: "7-day unstake",
    icon: TrendingUp,
    color: "text-yellow-500",
  },
  lock: {
    type: "lock",
    name: "Lock Pool",
    description: "High-risk staking with flexible lock periods and yield boosts",
    riskLevel: "High",
    apy: 12.5,
    liquidityStatus: "1 month to 1 year lock",
    icon: Lock,
    color: "text-red-500",
  },
}

export function PoolSelector() {
  const [selectedPool, setSelectedPool] = useState<PoolType>("basic")
  const { toast } = useToast()

  const handlePoolChange = (value: string) => {
    const poolType = value as PoolType
    setSelectedPool(poolType)

    // Update global state or context here
    window.localStorage.setItem("selectedPool", poolType)

    toast({
      title: `${poolData[poolType].name} selected`,
      description: `You've selected the ${poolData[poolType].name} for staking`,
    })
  }

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Select Staking Pool</h2>
      <Tabs defaultValue="basic" value={selectedPool} onValueChange={handlePoolChange} className="w-full">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="basic">Basic Pool</TabsTrigger>
          <TabsTrigger value="lending">Lending Pool</TabsTrigger>
          <TabsTrigger value="lock">Lock Pool</TabsTrigger>
        </TabsList>
        {Object.values(poolData).map((pool) => (
          <TabsContent key={pool.type} value={pool.type} className="mt-4">
            <Card>
              <CardContent className="pt-6">
                <div className="flex items-start gap-4">
                  <div
                    className={cn(
                      "rounded-full p-2",
                      pool.type === "basic"
                        ? "bg-green-100 dark:bg-green-900/30"
                        : pool.type === "lending"
                          ? "bg-yellow-100 dark:bg-yellow-900/30"
                          : "bg-red-100 dark:bg-red-900/30",
                    )}
                  >
                    <pool.icon className={cn("h-6 w-6", pool.color)} />
                  </div>
                  <div className="space-y-1">
                    <h3 className="text-lg font-medium">{pool.name}</h3>
                    <p className="text-sm text-muted-foreground">{pool.description}</p>
                    <div className="mt-4 grid grid-cols-3 gap-4">
                      <div>
                        <p className="text-sm font-medium">Risk Level</p>
                        <p
                          className={cn(
                            "text-sm",
                            pool.riskLevel === "Low"
                              ? "text-green-500"
                              : pool.riskLevel === "Medium"
                                ? "text-yellow-500"
                                : "text-red-500",
                          )}
                        >
                          {pool.riskLevel}
                        </p>
                      </div>
                      <div>
                        <p className="text-sm font-medium">APY</p>
                        <p className="text-sm text-primary">
                          {pool.apy}%{pool.type === "lock" && <span> - 25%</span>}
                        </p>
                      </div>
                      <div>
                        <p className="text-sm font-medium">Liquidity</p>
                        <p className="text-sm">{pool.liquidityStatus}</p>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        ))}
      </Tabs>
    </div>
  )
}
