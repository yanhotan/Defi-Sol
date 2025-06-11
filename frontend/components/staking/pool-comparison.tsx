import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Shield, TrendingUp, Lock, Check, X } from "lucide-react"

export function PoolComparison() {
  const comparisonData = [
    { feature: "Risk Level", basic: "Low", lending: "Medium", lock: "High" },
    { feature: "APY", basic: "5.2%", lending: "7.8%", lock: "12.5% - 25%" },
    { feature: "Unstaking Period", basic: "Instant", lending: "7 days", lock: "1 month - 1 year" },
    { feature: "Minimum Stake", basic: "0.1 SOL", lending: "1 SOL", lock: "5 SOL" },
    { feature: "Early Withdrawal", basic: "Yes", lending: "Yes (fee)", lock: "No" },
    { feature: "Compounding", basic: "Daily", lending: "Daily", lock: "At term end" },
    { feature: "Yield Source", basic: "Staking", lending: "Staking + Lending", lock: "Staking + Yield Boost" },
  ]

  return (
    <Card>
      <CardHeader>
        <CardTitle>Pool Comparison</CardTitle>
        <CardDescription>Compare the features and benefits of each staking pool</CardDescription>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[200px]">Feature</TableHead>
              <TableHead>
                <div className="flex items-center gap-2">
                  <Shield className="h-4 w-4 text-green-500" />
                  <span>Basic Pool</span>
                </div>
              </TableHead>
              <TableHead>
                <div className="flex items-center gap-2">
                  <TrendingUp className="h-4 w-4 text-yellow-500" />
                  <span>Lending Pool</span>
                </div>
              </TableHead>
              <TableHead>
                <div className="flex items-center gap-2">
                  <Lock className="h-4 w-4 text-red-500" />
                  <span>Lock Pool</span>
                </div>
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {comparisonData.map((row) => (
              <TableRow key={row.feature}>
                <TableCell className="font-medium">{row.feature}</TableCell>
                <TableCell>
                  {row.basic === "Yes" ? (
                    <Check className="h-4 w-4 text-green-500" />
                  ) : row.basic === "No" ? (
                    <X className="h-4 w-4 text-red-500" />
                  ) : (
                    row.basic
                  )}
                </TableCell>
                <TableCell>
                  {row.lending === "Yes" ? (
                    <Check className="h-4 w-4 text-green-500" />
                  ) : row.lending === "No" ? (
                    <X className="h-4 w-4 text-red-500" />
                  ) : (
                    row.lending
                  )}
                </TableCell>
                <TableCell>
                  {row.lock === "Yes" ? (
                    <Check className="h-4 w-4 text-green-500" />
                  ) : row.lock === "No" ? (
                    <X className="h-4 w-4 text-red-500" />
                  ) : (
                    row.lock
                  )}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  )
}
