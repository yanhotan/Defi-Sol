"use client"

import type React from "react"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useToast } from "@/hooks/use-toast"
import type { ProductType } from "@/components/products/risk-pool-selector"

// Lock period options in days
const lockPeriodOptions = [
  { value: 30, label: "1 Month", boost: 0 },
  { value: 90, label: "3 Months", boost: 25 },
  { value: 180, label: "6 Months", boost: 50 },
  { value: 270, label: "9 Months", boost: 75 },
  { value: 365, label: "1 Year", boost: 100 },
]

interface StakingFormProps {
  productType: ProductType
}

export function StakingForm({ productType }: StakingFormProps) {
  const [amount, setAmount] = useState<number>(1)
  const [balance, setBalance] = useState<number>(30)
  const [isStaking, setIsStaking] = useState(false)
  const [selectedRisk, setSelectedRisk] = useState<string>("low")
  const [lockPeriodIndex, setLockPeriodIndex] = useState(0)
  const [autoCompound, setAutoCompound] = useState(true)
  const [inputType, setInputType] = useState<"sol" | "msol">("sol")
  const { toast } = useToast()

  // Set balance based on product type
  useEffect(() => {
    if (productType === "msol") {
      setBalance(inputType === "sol" ? 30 : 25)
    } else if (productType === "usdc") {
      setBalance(1500)
    }
  }, [productType, inputType])

  // Get the selected risk level from localStorage
  useEffect(() => {
    const savedRisk = window.localStorage.getItem(`${productType}:riskLevel`)
    if (savedRisk) {
      setSelectedRisk(savedRisk)
    }
  }, [productType])

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
        description: `You don't have enough ${inputType === "sol" ? "SOL" : productType === "usdc" ? "USDC" : "mSOL"}`,
        variant: "destructive",
      })
      return
    }

    // Check minimum stake requirements
    const minimumStake = productType === "usdc" ? 10 : 0.1
    if (amount < minimumStake) {
      toast({
        title: "Below minimum stake",
        description: `The minimum stake is ${minimumStake} ${productType === "usdc" ? "USDC" : inputType === "sol" ? "SOL" : "mSOL"}`,
        variant: "destructive",
      })
      return
    }

    setIsStaking(true)

    // Simulate staking process
    setTimeout(() => {
      let successMessage = ""

      if (productType === "msol") {
        const lockPeriod = selectedRisk === "high" ? lockPeriodOptions[lockPeriodIndex].label : "none"
        successMessage = `You have successfully staked ${amount} ${inputType === "sol" ? "SOL" : "mSOL"} in the ${selectedRisk} risk pool`
        if (lockPeriod !== "none") {
          successMessage += ` for ${lockPeriod}`
        }
      } else if (productType === "usdc") {
        successMessage = `You have successfully staked ${amount} USDC`
      }

      toast({
        title: "Staking successful",
        description: successMessage,
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

  // Calculate estimated rewards based on product type and risk level
  const calculateEstimatedRewards = () => {
    let apy = 5.2 // default APY

    if (productType === "msol") {
      if (selectedRisk === "low") apy = 5.2
      else if (selectedRisk === "medium") apy = 7.8
      else if (selectedRisk === "high") {
        apy = 12.5
        // Apply lock period multiplier for high risk
        const multiplier = 1 + lockPeriodOptions[lockPeriodIndex].boost / 100
        apy *= multiplier
      }
    } else if (productType === "usdc") {
      apy = 5.5
    }

    const dailyRate = apy / 365 / 100
    return (amount * dailyRate).toFixed(6)
  }

  // Get estimated APY with any bonuses
  const getAdjustedApy = () => {
    let apy = 5.2 // default APY

    if (productType === "msol") {
      if (selectedRisk === "low") apy = 5.2
      else if (selectedRisk === "medium") apy = 7.8
      else if (selectedRisk === "high") {
        apy = 12.5
        // Apply lock period multiplier for high risk
        const multiplier = 1 + lockPeriodOptions[lockPeriodIndex].boost / 100
        apy *= multiplier
      }
    } else if (productType === "usdc") {
      apy = 5.5
    }

    return apy.toFixed(1)
  }

  // Get the current lock period in days
  const getCurrentLockPeriod = () => {
    if (productType === "msol" && selectedRisk === "high") {
      return lockPeriodOptions[lockPeriodIndex].value
    }
    return selectedRisk === "medium" ? 7 : 0
  }

  // Get the current lock period label
  const getCurrentLockPeriodLabel = () => {
    if (productType === "msol" && selectedRisk === "high") {
      return lockPeriodOptions[lockPeriodIndex].label
    }
    return selectedRisk === "medium" ? "7 days" : "None"
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>
          {productType === "msol" ? "Stake SOL for mSOL" : productType === "usdc" ? "Stake USDC" : "Stake mSOL-USDC"}
        </CardTitle>
        <CardDescription>
          {productType === "msol"
            ? "Stake your SOL to receive mSOL and earn rewards"
            : productType === "usdc"
              ? "Stake your USDC to earn stable yields"
              : "Stake both mSOL and USDC to earn enhanced yields"}
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {productType === "msol" && (
          <Tabs value={inputType} onValueChange={(value) => setInputType(value as "sol" | "msol")} className="w-full">
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="sol">Stake SOL</TabsTrigger>
              <TabsTrigger value="msol">Stake mSOL</TabsTrigger>
            </TabsList>
          </Tabs>
        )}

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label htmlFor="amount">Amount</Label>
            <span className="text-xs text-muted-foreground">
              Balance: {balance} {productType === "usdc" ? "USDC" : inputType === "sol" ? "SOL" : "mSOL"}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <Input
              id="amount"
              type="number"
              value={amount}
              onChange={handleInputChange}
              min={0}
              max={balance}
              step={productType === "usdc" ? 1 : 0.1}
            />
            <Button variant="outline" size="sm" onClick={handleMaxClick}>
              Max
            </Button>
          </div>
        </div>
        <div className="space-y-2">
          <div className="flex justify-between">
            <span className="text-sm">0 {productType === "usdc" ? "USDC" : inputType === "sol" ? "SOL" : "mSOL"}</span>
            <span className="text-sm">
              {balance} {productType === "usdc" ? "USDC" : inputType === "sol" ? "SOL" : "mSOL"}
            </span>
          </div>
          <Slider
            value={[amount]}
            max={balance}
            step={productType === "usdc" ? 1 : 0.1}
            onValueChange={handleSliderChange}
          />
        </div>

        {productType === "msol" && selectedRisk === "high" && (
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

        {selectedRisk !== "high" && (
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
            <span className="text-sm font-medium">
              {productType === "msol" && inputType === "sol"
                ? `${(amount * 0.98).toFixed(2)} mSOL`
                : `${amount} ${productType === "usdc" ? "stUSDC" : "stMSOL"}`}
            </span>
          </div>
          <div className="mt-2 flex justify-between">
            <span className="text-sm">Annual percentage yield</span>
            <span className="text-sm font-medium">{getAdjustedApy()}%</span>
          </div>
          <div className="mt-2 flex justify-between">
            <span className="text-sm">Estimated daily rewards</span>
            <span className="text-sm font-medium">
              {calculateEstimatedRewards()} {productType === "usdc" ? "USDC" : "SOL"}
            </span>
          </div>
          {selectedRisk === "medium" && (
            <div className="mt-2 flex justify-between">
              <span className="text-sm">Unstaking period</span>
              <span className="text-sm font-medium">7 days</span>
            </div>
          )}
          {selectedRisk === "high" && (
            <div className="mt-2 flex justify-between">
              <span className="text-sm">Lock period</span>
              <span className="text-sm font-medium">{getCurrentLockPeriod()} days</span>
            </div>
          )}
        </div>
      </CardContent>
      <CardFooter>
        <Button
          className="w-full bg-solana-gradient hover:opacity-90"
          onClick={handleStake}
          disabled={amount <= 0 || amount > balance || isStaking}
        >
          {isStaking ? "Staking..." : `Stake ${productType === "usdc" ? "USDC" : inputType === "sol" ? "SOL" : "mSOL"}`}
        </Button>
      </CardFooter>
    </Card>
  )
}
