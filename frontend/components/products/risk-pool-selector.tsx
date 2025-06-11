"use client"

import type React from "react"

import { useState, useEffect } from "react"
import { Card, CardContent } from "@/components/ui/card"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import { Shield, TrendingUp, Lock } from "lucide-react"
import { useToast } from "@/hooks/use-toast"
import { cn } from "@/lib/utils"

export type RiskLevel = "low" | "medium" | "high"
export type ProductType = "msol" | "usdc" | "msol-usdc"

interface RiskPoolInfo {
  level: RiskLevel
  name: string
  description: string
  apy: number
  lockPeriod: string
  icon: React.ElementType
  color: string
  bgColor: string
}

const getRiskPoolData = (productType: ProductType): Record<RiskLevel, RiskPoolInfo> => {
  if (productType === "msol") {
    return {
      low: {
        level: "low",
        name: "Low Risk",
        description: "Safe staking with instant unstake capability",
        apy: 5.2,
        lockPeriod: "None",
        icon: Shield,
        color: "text-solana-teal",
        bgColor: "bg-solana-teal/10",
      },
      medium: {
        level: "medium",
        name: "Medium Risk",
        description: "Balanced risk-reward with 7-day unstaking period",
        apy: 7.8,
        lockPeriod: "7 days",
        icon: TrendingUp,
        color: "text-solana-blue",
        bgColor: "bg-solana-blue/10",
      },
      high: {
        level: "high",
        name: "High Risk",
        description: "Higher yields with flexible lock periods",
        apy: 12.5,
        lockPeriod: "30-365 days",
        icon: Lock,
        color: "text-solana-purple",
        bgColor: "bg-solana-purple/10",
      },
    }
  } else if (productType === "msol-usdc") {
    return {
      low: {
        level: "low",
        name: "Low Risk",
        description: "Safe dual asset staking with instant unstake",
        apy: 8.5,
        lockPeriod: "None",
        icon: Shield,
        color: "text-solana-teal",
        bgColor: "bg-solana-teal/10",
      },
      medium: {
        level: "medium",
        name: "Medium Risk",
        description: "LP enabled with 7-day unstaking period",
        apy: 12.8,
        lockPeriod: "7 days",
        icon: TrendingUp,
        color: "text-solana-blue",
        bgColor: "bg-solana-blue/10",
      },
      high: {
        level: "high",
        name: "High Risk",
        description: "Maximum yield with LP and lock periods",
        apy: 18.5,
        lockPeriod: "30-365 days",
        icon: Lock,
        color: "text-solana-purple",
        bgColor: "bg-solana-purple/10",
      },
    }
  }

  // Default fallback (should never happen)
  return {
    low: {
      level: "low",
      name: "Low Risk",
      description: "Safe staking with instant unstake capability",
      apy: 5.2,
      lockPeriod: "None",
      icon: Shield,
      color: "text-solana-teal",
      bgColor: "bg-solana-teal/10",
    },
    medium: {
      level: "medium",
      name: "Medium Risk",
      description: "Balanced risk-reward with 7-day unstaking period",
      apy: 7.8,
      lockPeriod: "7 days",
      icon: TrendingUp,
      color: "text-solana-blue",
      bgColor: "bg-solana-blue/10",
    },
    high: {
      level: "high",
      name: "High Risk",
      description: "Higher yields with flexible lock periods",
      apy: 12.5,
      lockPeriod: "30-365 days",
      icon: Lock,
      color: "text-solana-purple",
      bgColor: "bg-solana-purple/10",
    },
  }
}

interface RiskPoolSelectorProps {
  productType: ProductType
}

export function RiskPoolSelector({ productType }: RiskPoolSelectorProps) {
  const [selectedRisk, setSelectedRisk] = useState<RiskLevel>("low")
  const { toast } = useToast()
  const riskPoolData = getRiskPoolData(productType)

  const handleRiskChange = (value: string) => {
    const riskLevel = value as RiskLevel
    setSelectedRisk(riskLevel)

    // Update global state or context here
    window.localStorage.setItem(`${productType}:riskLevel`, riskLevel)

    toast({
      title: `${riskPoolData[riskLevel].name} selected`,
      description: `You've selected the ${riskPoolData[riskLevel].name} pool for ${productType === "msol" ? "mSOL" : "mSOL-USDC"} staking`,
    })
  }

  // Load saved risk level from localStorage
  useEffect(() => {
    const savedRisk = window.localStorage.getItem(`${productType}:riskLevel`) as RiskLevel
    if (savedRisk && riskPoolData[savedRisk]) {
      setSelectedRisk(savedRisk)
    }
  }, [productType, riskPoolData])

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Select Risk Level</h2>
      <Tabs defaultValue="low" value={selectedRisk} onValueChange={handleRiskChange} className="w-full">
        <TabsList className="grid w-full grid-cols-3 bg-muted/50">
          <TabsTrigger
            value="low"
            className={cn(
              "data-[state=active]:bg-solana-teal/10 data-[state=active]:text-solana-teal data-[state=active]:shadow-none",
            )}
          >
            Low Risk
          </TabsTrigger>
          <TabsTrigger
            value="medium"
            className={cn(
              "data-[state=active]:bg-solana-blue/10 data-[state=active]:text-solana-blue data-[state=active]:shadow-none",
            )}
          >
            Medium Risk
          </TabsTrigger>
          <TabsTrigger
            value="high"
            className={cn(
              "data-[state=active]:bg-solana-purple/10 data-[state=active]:text-solana-purple data-[state=active]:shadow-none",
            )}
          >
            High Risk
          </TabsTrigger>
        </TabsList>
        {Object.values(riskPoolData).map((pool) => (
          <TabsContent key={pool.level} value={pool.level} className="mt-4">
            <Card>
              <CardContent className="pt-6">
                <div className="flex items-start gap-4">
                  <div className={cn("rounded-full p-2", pool.bgColor)}>
                    <pool.icon className={cn("h-6 w-6", pool.color)} />
                  </div>
                  <div className="space-y-1">
                    <h3 className="text-lg font-medium">{pool.name}</h3>
                    <p className="text-sm text-muted-foreground">{pool.description}</p>
                    <div className="mt-4 grid grid-cols-3 gap-4">
                      <div>
                        <p className="text-sm font-medium">Risk Level</p>
                        <p className={cn("text-sm", pool.color)}>{pool.name}</p>
                      </div>
                      <div>
                        <p className="text-sm font-medium">APY</p>
                        <p className="text-sm text-primary">{pool.apy}%</p>
                      </div>
                      <div>
                        <p className="text-sm font-medium">Lock Period</p>
                        <p className="text-sm">{pool.lockPeriod}</p>
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
