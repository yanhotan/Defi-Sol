"use client"

import type React from "react"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { useToast } from "@/hooks/use-toast"
import { type PoolType, poolData } from "@/components/staking/pool-selector"

// Lock period options in days
const lockPeriodOptions = [
  { value: 30, label: "1 Month", boost: 0 },
  { value: 90, label: "3 Months", boost: 25 },
  { value: 180, label: "6 Months", boost: 50 },
  { value: 270, label: "9 Months", boost: 75 },
  { value: 365, label: "1 Year", boost: 100 },
]

export function StakingForm() {
  const [amount, setAmount] = useState<number>(1)
  const [balance] = useState<number>(30)
  const [isStaking, setIsStaking] = useState(false)
  const [selectedPool, setSelectedPool] = useState<PoolType>("basic")
  const [lockPeriodIndex, setLockPeriodIndex] = useState(0)
  const [autoCompound, setAutoCompound] = useState(true)
  const { toast } = useToast()

  // Get the selected pool from localStorage
  useEffect(() => {
    const storedPool = window.localStorage.getItem("selectedPool") as PoolType
    if (storedPool) {
      setSelectedPool(storedPool)
    }
  }, [])

  const handleStake = () => {
    if (amount <= 0) {
      toast({
        title: "Invalid amount",
        description: "Please enter a valid amount to stake",
        variant: "destructive",
      })
      return
    }

    if (amount > balance) {
      toast({
        title: "Insufficient balance",
        description: "You don't have enough SOL to stake this amount",
        variant: "destructive",
      })
      return
    }

    // Check minimum stake requirements
    const minimumStake = selectedPool === "basic" ? 0.1 : selectedPool === "lending" ? 1 : 5
    if (amount < minimumStake) {
      toast({
        title: "Below minimum stake",
        description: `The minimum stake for ${poolData[selectedPool].name} is ${minimumStake} SOL`,
        variant: "destructive",
      })
      return
    }

    setIsStaking(true)

    // Simulate staking process
    setTimeout(() => {
      const lockPeriod =
        selectedPool === "lock"
          ? lockPeriodOptions[lockPeriodIndex].label
          : selectedPool === "lending"
            ? "7 days"
            : "none"

      toast({
        title: "Staking successful",
        description: `You have successfully staked ${amount} SOL in the ${poolData[selectedPool].name}${
          lockPeriod !== "none" ? ` for ${lockPeriod}` : ""
        }`,
      })
      setIsStaking(false)
    }, 2000)
  }

  const handleSliderChange = (value: number[]) => {
    setAmount(value[0])
  }

  const handleLockPeriodChange = (value: number[]) => {
    setLockPeriodIndex(value[0])
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = Number.parseFloat(e.target.value)
    if (!isNaN(value)) {
      setAmount(value > balance ? balance : value)
    } else {
      setAmount(0)
    }
  }

  const handleMaxClick = () => {
    setAmount(balance)
  }

  // Calculate estimated rewards based on pool type and amount
  const calculateEstimatedRewards = () => {
    const apy = poolData[selectedPool].apy
    let multiplier = 1

    // Apply lock period multiplier for lock pool
    if (selectedPool === "lock") {
      multiplier = 1 + lockPeriodOptions[lockPeriodIndex].boost / 100
    }

    const adjustedApy = apy * multiplier
    const dailyRate = adjustedApy / 365 / 100
    return (amount * dailyRate).toFixed(6)
  }

  // Get estimated APY with any bonuses
  const getAdjustedApy = () => {
    const baseApy = poolData[selectedPool].apy
    let multiplier = 1

    // Apply lock period multiplier for lock pool
    if (selectedPool === "lock") {
      multiplier = 1 + lockPeriodOptions[lockPeriodIndex].boost / 100
    }

    return (baseApy * multiplier).toFixed(1)
  }

  // Get the current lock period in days
  const getCurrentLockPeriod = () => {
    if (selectedPool === "lock") {
      return lockPeriodOptions[lockPeriodIndex].value
    }
    return selectedPool === "lending" ? 7 : 0
  }

  // Get the current lock period label
  const getCurrentLockPeriodLabel = () => {
    if (selectedPool === "lock") {
      return lockPeriodOptions[lockPeriodIndex].label
    }
    return selectedPool === "lending" ? "7 days" : "None"
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Stake SOL</CardTitle>
        <CardDescription>Stake your SOL tokens in the {poolData[selectedPool].name}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label htmlFor="amount">Amount</Label>
            <span className="text-xs text-muted-foreground">Balance: {balance} SOL</span>
          </div>
          <div className="flex items-center gap-2">
            <Input
              id="amount"
              type="number"
              value={amount}
              onChange={handleInputChange}
              min={0}
              max={balance}
              step={0.1}
            />
            <Button variant="outline" size="sm" onClick={handleMaxClick}>
              Max
            </Button>
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex justify-between">
            <span className="text-sm">0 SOL</span>
            <span className="text-sm">{balance} SOL</span>
          </div>
          <Slider value={[amount]} max={balance} step={0.1} onValueChange={handleSliderChange} />
        </div>

        {selectedPool === "lock" && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label htmlFor="lock-period">Lock Period</Label>
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">{getCurrentLockPeriodLabel()}</span>
                {lockPeriodOptions[lockPeriodIndex].boost > 0 && (
                  <span className="rounded-full bg-green-100 px-2 py-0.5 text-xs font-medium text-green-800 dark:bg-green-900/30 dark:text-green-400">
                    +{lockPeriodOptions[lockPeriodIndex].boost}% APY
                  </span>
                )}
              </div>
            </div>
            <Slider
              id="lock-period"
              value={[lockPeriodIndex]}
              max={lockPeriodOptions.length - 1}
              step={1}
              onValueChange={handleLockPeriodChange}
            />
            <div className="flex justify-between text-xs text-muted-foreground">
              {lockPeriodOptions.map((option, index) => (
                <div key={option.value} className={index === lockPeriodIndex ? "font-medium text-primary" : ""}>
                  {option.label}
                </div>
              ))}
            </div>
          </div>
        )}

        {selectedPool !== "lock" && (
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="auto-compound">Auto-Compound</Label>
              <p className="text-xs text-muted-foreground">Automatically reinvest your rewards</p>
            </div>
            <Switch id="auto-compound" checked={autoCompound} onCheckedChange={setAutoCompound} />
          </div>
        )}

        <div className="rounded-lg bg-muted p-4">
          <div className="flex justify-between">
            <span className="text-sm">You will receive</span>
            <span className="text-sm font-medium">{amount} mSOL</span>
          </div>
          <div className="mt-2 flex justify-between">
            <span className="text-sm">Annual percentage yield</span>
            <span className="text-sm font-medium">{getAdjustedApy()}%</span>
          </div>
          <div className="mt-2 flex justify-between">
            <span className="text-sm">Estimated daily rewards</span>
            <span className="text-sm font-medium">{calculateEstimatedRewards()} SOL</span>
          </div>
          {selectedPool === "lending" && (
            <div className="mt-2 flex justify-between">
              <span className="text-sm">Unstaking period</span>
              <span className="text-sm font-medium">7 days</span>
            </div>
          )}
          {selectedPool === "lock" && (
            <div className="mt-2 flex justify-between">
              <span className="text-sm">Lock period</span>
              <span className="text-sm font-medium">{getCurrentLockPeriod()} days</span>
            </div>
          )}
        </div>
      </CardContent>
      <CardFooter>
        <Button className="w-full" onClick={handleStake} disabled={amount <= 0 || amount > balance || isStaking}>
          {isStaking ? "Staking..." : "Stake SOL"}
        </Button>
      </CardFooter>
    </Card>
  )
}
