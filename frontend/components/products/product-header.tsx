interface ProductHeaderProps {
  title: string
  description: string
  productType: string
}

export function ProductHeader({ title, description, productType }: ProductHeaderProps) {
  return (
    <div className="flex flex-col gap-2">
      <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
      <p className="text-muted-foreground">{description}</p>
    </div>
  )
}
