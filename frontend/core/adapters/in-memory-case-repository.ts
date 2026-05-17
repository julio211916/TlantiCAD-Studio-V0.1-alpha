import type { DentalCase } from '../domain/entities'
import type { CaseRepository } from '../ports/case-repository'

export class InMemoryCaseRepository implements CaseRepository {
  private readonly cases = new Map<string, DentalCase>()

  constructor(seed: readonly DentalCase[] = []) {
    seed.forEach((dentalCase) => {
      this.cases.set(dentalCase.id, { ...dentalCase, assets: [...dentalCase.assets] })
    })
  }

  async findById(caseId: string): Promise<DentalCase | null> {
    const dentalCase = this.cases.get(caseId)
    return dentalCase ? { ...dentalCase, assets: [...dentalCase.assets] } : null
  }

  async list(): Promise<readonly DentalCase[]> {
    return Array.from(this.cases.values()).map((dentalCase) => ({ ...dentalCase, assets: [...dentalCase.assets] }))
  }

  async save(dentalCase: DentalCase): Promise<DentalCase> {
    const nextCase = { ...dentalCase, assets: [...dentalCase.assets] }
    this.cases.set(dentalCase.id, nextCase)
    return { ...nextCase, assets: [...nextCase.assets] }
  }
}
