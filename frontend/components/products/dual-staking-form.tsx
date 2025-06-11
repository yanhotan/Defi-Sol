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
import { ArrowDown, RefreshCw } from "lucide-react"

// Lock period options in days
const lockPeriodOptions = [
  { value: 30, label: "1 Month", boost: 0 },
  { value: 90, label: "3 Months", boost: 25 },
  { value: 180, label: "6 Months", boost: 50 },
  { value: 270, label: "9 Months", boost: 75 },
  { value: 365, label: "1 Year", boost: 100 },
]

export function DualStakingForm() {
  const [msolAmount, setMsolAmount] = useState<number>(1)
  const [usdcAmount, setUsdcAmount] = useState<number>(100)
  const [msolBalance, setMsolBalance] = useState<number>(5)
  const [usdcBalance, setUsdcBalance] = useState<number>(500)
  const [solBalance, setSolBalance] = useState<number>(10)
  const [isStaking, setIsStaking] = useState(false)
  const [selectedRisk, setSelectedRisk] = useState<string>("low")
  const [lockPeriodIndex, setLockPeriodIndex] = useState(0)
  const [lpEnabled, setLpEnabled] = useState(false)
  const [inputType, setInputType] = useState<"separate" | "sol">("separate")
  const [solAmount, setSolAmount] = useState<number>(5)
  const { toast } = useToast()

  // Get the selected risk level from localStorage
  useEffect(() => {
    const savedRisk = window.localStorage.getItem("msol-usdc:riskLevel")
    if (savedRisk) {
      setSelectedRisk(savedRisk)
    }

    // Enable LP by default for medium and high risk
    if (savedRisk === "medium" || savedRisk === "high") {
      setLpEnabled(true)
    }
  }, [])

  const handleStake = () => {
    if (inputType === "separate") {
      if (msolAmount <= 0 || usdcAmount <= 0) {
        toast({
          title: "Invalid amount",
          description: "Please enter valid amounts for both mSOL and USDC",
          variant: "destructive",
        })
        return
      }

      if (msolAmount > msolBalance || usdcAmount > usdcBalance) {
        toast({
          title: "Insufficient balance",
          description: "You don't have enough mSOL or USDC",
          variant: "destructive",
        })
        return
      }
    } else {
      if (solAmount <= 0) {
        toast({
          title: "Invalid amount",
          description: "Please enter a valid amount of SOL",
          variant: "destructive",
        })
        return
      }

      if (solAmount > solBalance) {
        toast({
          title: "Insufficient balance",
          description: "You don't have enough SOL",
          variant: "destructive",
        })
        return
      }
    }

    setIsStaking(true)

    // Simulate staking process
    setTimeout(() => {
      let successMessage = ""

      if (inputType === "separate") {
        successMessage = `You have successfully staked ${msolAmount} mSOL and ${usdcAmount} USDC in the ${selectedRisk} risk pool`
      } else {
        successMessage = `You have successfully swapped ${solAmount} SOL to ${(solAmount * 0.49).toFixed(2)} mSOL and ${(solAmount * 70).toFixed(2)} USDC and staked in the ${selectedRisk} risk pool`
      }

      if (lpEnabled) {
        successMessage += " with LP enabled"
      }

      if (selectedRisk === "high") {
        successMessage += ` for ${lockPeriodOptions[lockPeriodIndex].label}`
      }

      toast({
        title: "Staking successful",
        description: successMessage,
      })
      setIsStaking(false)
    }, 2000)
  }

  const handleMsolSliderChange = (value: number[]) => {
    setMsolAmount(value[0])
  }

  const handleUsdcSliderChange = (value: number[]) => {
    setUsdcAmount(value[0])
  }

  const handleSolSliderChange = (value: number[]) => {
    setSolAmount(value[0])
  }

  const handleLockPeriodChange = (value: number[]) => {
    setLockPeriodIndex(value[0])
  }

  const handleMsolInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = Number.parseFloat(e.target.value)
    if (!isNaN(value)) {
      setMsolAmount(value > msolBalance ? msolBalance : value)
    } else {
      setMsolAmount(0)
    }
  }

  const handleUsdcInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = Number.parseFloat(e.target.value)
    if (!isNaN(value)) {
      setUsdcAmount(value > usdcBalance ? usdcBalance : value)
    } else {
      setUsdcAmount(0)
    }
  }

  const handleSolInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = Number.parseFloat(e.target.value)
    if (!isNaN(value)) {
      setSolAmount(value > solBalance ? solBalance : value)
    } else {
      setSolAmount(0)
    }
  }

  const handleMsolMaxClick = () => {
    setMsolAmount(msolBalance)
  }

  const handleUsdcMaxClick = () => {
    setUsdcAmount(usdcBalance)
  }

  const handleSolMaxClick = () => {
    setSolAmount(solBalance)
  }

  // Calculate estimated rewards based on risk level
  const calculateEstimatedRewards = () => {
    let apy = 8.5 // default APY for low risk

    if (selectedRisk === "low") apy = 8.5
    else if (selectedRisk === "medium") apy = 12.8
    else if (selectedRisk === "high") {
      apy = 18.5
      // Apply lock period multiplier for high risk
      const multiplier = 1 + lockPeriodOptions[lockPeriodIndex].boost / 100
      apy *= multiplier
    }

    // Add LP bonus if enabled
    if (lpEnabled && (selectedRisk === "medium" || selectedRisk === "high")) {
      apy += 2.0
    }

    const dailyRate = apy / 365 / 100

    // Calculate total value in USD
    const totalValueUsd = inputType === "separate" ? msolAmount * 142.87 + usdcAmount : solAmount * 142.87

    return (totalValueUsd * dailyRate).toFixed(2)
  }

  // Get estimated APY with any bonuses
  const getAdjustedApy = () => {
    let apy = 8.5 // default APY for low risk

    if (selectedRisk === "low") apy = 8.5
    else if (selectedRisk === "medium") apy = 12.8
    else if (selectedRisk === "high") {
      apy = 18.5
      // Apply lock period multiplier for high risk
      const multiplier = 1 + lockPeriodOptions[lockPeriodIndex].boost / 100
      apy *= multiplier
    }

    // Add LP bonus if enabled
    if (lpEnabled && (selectedRisk === "medium" || selectedRisk === "high")) {
      apy += 2.0
    }

    return apy.toFixed(1)
  }

  // Get the current lock period in days
  const getCurrentLockPeriod = () => {
    if (selectedRisk === "high") {
      return lockPeriodOptions[lockPeriodIndex].value
    }
    return selectedRisk === "medium" ? 7 : 0
  }

  // Get the current lock period label
  const getCurrentLockPeriodLabel = () => {
    if (selectedRisk === "high") {
      return lockPeriodOptions[lockPeriodIndex].label
    }
    return selectedRisk === "medium" ? "7 days" : "None"
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Stake mSOL-USDC</CardTitle>
        <CardDescription>Stake both mSOL and USDC to earn enhanced yields</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Tabs value={inputType} onValueChange={(value) => setInputType(value as "separate" | "sol")} className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="separate">Stake mSOL & USDC</TabsTrigger>
            <TabsTrigger value="sol">Swap SOL & Stake</TabsTrigger>
          </TabsList>
        </Tabs>

        {inputType === "separate" ? (
          <>
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="msol-amount">mSOL Amount</Label>
                <span className="text-xs text-muted-foreground">Balance: {msolBalance} mSOL</span>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  id="msol-amount"
                  type="number"
                  value={msolAmount}
                  onChange={handleMsolInputChange}
                  min={0}
                  max={msolBalance}
                  step={0.1}
                />
                <Button variant="outline" size="sm" onClick={handleMsolMaxClick}>
                  Max
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm">0 mSOL</span>
                <span className="text-sm">{msolBalance} mSOL</span>
              </div>
              <Slider value={[msolAmount]} max={msolBalance} step={0.1} onValueChange={handleMsolSliderChange} />
            </div>

            <div className="flex justify-center">
              <div className="rounded-full bg-muted p-2">
                <ArrowDown className="h-4 w-4" />
              </div>
            </div>

            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="usdc-amount">USDC Amount</Label>
                <span className="text-xs text-muted-foreground">Balance: {usdcBalance} USDC</span>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  id="usdc-amount"
                  type="number"
                  value={usdcAmount}
                  onChange={handleUsdcInputChange}
                  min={0}
                  max={usdcBalance}
                  step={1}
                />
                <Button variant="outline" size="sm" onClick={handleUsdcMaxClick}>
                  Max
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm">0 USDC</span>
                <span className="text-sm">{usdcBalance} USDC</span>
              </div>
              <Slider value={[usdcAmount]} max={usdcBalance} step={1} onValueChange={handleUsdcSliderChange} />
            </div>
          </>
        ) : (
          <div className="space-y-4">
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="sol-amount">SOL Amount</Label>
                <span className="text-xs text-muted-foreground">Balance: {solBalance} SOL</span>
              </div>
              <div className="flex items-center gap-2">
                <Input
                  id="sol-amount"
                  type="number"
                  value={solAmount}
                  onChange={handleSolInputChange}
                  min={0}
                  max={solBalance}
                  step={0.1}
                />
                <Button variant="outline" size="sm" onClick={handleSolMaxClick}>
                  Max
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm">0 SOL</span>
                <span className="text-sm">{solBalance} SOL</span>
              </div>
              <Slider value={[solAmount]} max={solBalance} step={0.1} onValueChange={handleSolSliderChange} />
            </div>

            <div className="flex justify-center">
              <div className="rounded-full bg-muted p-2">
                <RefreshCw className="h-4 w-4" />
              </div>
            </div>

            <div className="rounded-lg bg-muted p-4 space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-sm">You will receive:</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">mSOL:</span>
                <span className="text-sm font-medium">{(solAmount * 0.49).toFixed(2)} mSOL</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm">USDC:</span>
                <span className="text-sm font-medium">{(solAmount * 70).toFixed(2)} USDC</span>
              </div>
              <div className="text-xs text-muted-foreground mt-2">Powered by Jupiter Swap</div>
            </div>
          </div>
        )}

        {(selectedRisk === "medium" || selectedRisk === "high") && (
          <div className="flex items-center justify-between">
            <div className="space-y-0.5">
              <Label htmlFor="lp-enabled">Enable LP</Label>
              <p className="text-xs text-muted-foreground">Provide liquidity to earn additional rewards</p>
            </div>
            <Switch id="lp-enabled" checked={lpEnabled} onCheckedChange={setLpEnabled} />
          </div>
        )}

        {selectedRisk === "high" && (
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

        <div className="rounded-lg bg-muted p-4">
          <div className="flex justify-between">
            <span className="text-sm">Total value</span>
            <span className="text-sm font-medium">
              $
              {inputType === "separate"
                ? (msolAmount * 142.87 + usdcAmount).toFixed(2)
                : (solAmount * 142.87).toFixed(2)}
            </span>
          </div>
          <div className="mt-2 flex justify-between">
            <span className="text-sm">Annual percentage yield</span>
            <span className="text-sm font-medium">{getAdjustedApy()}%</span>
          </div>
          <div className="mt-2 flex justify-between">
            <span className="text-sm">Estimated daily rewards</span>
            <span className="text-sm font-medium">${calculateEstimatedRewards()}</span>
          </div>
          {lpEnabled && (
            <div className="mt-2 flex justify-between">
              <span className="text-sm">LP rewards</span>
              <span className="text-sm font-medium">+2.0% APY</span>
            </div>
          )}
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
        <Button className="w-full bg-solana-gradient hover:opacity-90" onClick={handleStake} disabled={isStaking}>
          {isStaking ? "Staking..." : inputType === "separate" ? "Stake mSOL & USDC" : "Swap & Stake"}
        </Button>
      </CardFooter>
    </Card>
  )
}
