"use client"

import { useState, useEffect } from "react"
import { ArrowRight } from "lucide-react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Progress } from "@/components/ui/progress"
import Link from "next/link"

export function StakingOverview() {
  const [stakingData, setStakingData] = useState({
    active: 0,
    total: 0,
    validators: 0,
  })

  useEffect(() => {
    // Simulate fetching staking data
    setStakingData({
      active: 24.5,
      total: 30,
      validators: 3,
    })
  }, [])

  const progressPercentage = (stakingData.active / stakingData.total) * 100

  return (
    <Card>
      <CardHeader>
        <CardTitle>Staking Overview</CardTitle>
        <CardDescription>Your current staking positions and performance</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <div className="flex justify-between">
            <span className="text-sm">Active Stake</span>
            <span className="text-sm font-medium">
              {stakingData.active} / {stakingData.total} SOL
            </span>
          </div>
          <Progress value={progressPercentage} className="h-2" />
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Validators</p>
            <p className="text-lg font-medium">{stakingData.validators}</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Delegation Fee</p>
            <p className="text-lg font-medium">2%</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Unstaking Period</p>
            <p className="text-lg font-medium">~2-3 days</p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-muted-foreground">Compounding</p>
            <p className="text-lg font-medium">Automatic</p>
          </div>
        </div>
      </CardContent>
      <CardFooter>
        <Button asChild className="w-full">
          <Link href="/stake">
            Stake More SOL
            <ArrowRight className="ml-2 h-4 w-4" />
          </Link>
        </Button>
      </CardFooter>
    </Card>
  )
}
