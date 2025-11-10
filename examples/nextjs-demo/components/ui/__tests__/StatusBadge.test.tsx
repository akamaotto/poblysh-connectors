import { render, screen } from '@testing-library/react'
import { StatusBadge } from '../StatusBadge'

describe('StatusBadge', () => {
  it('renders with success status', () => {
    render(<StatusBadge status="success">Completed</StatusBadge>)

    const badge = screen.getByRole('status')
    expect(badge).toBeInTheDocument()
    expect(badge).toHaveTextContent('Completed')
    expect(badge).toHaveAttribute('aria-label', 'Status: success')
    expect(badge).toHaveClass('bg-green-100', 'text-green-800', 'border-green-200')
  })

  it('renders with error status', () => {
    render(<StatusBadge status="error">Failed</StatusBadge>)

    const badge = screen.getByRole('status')
    expect(badge).toBeInTheDocument()
    expect(badge).toHaveTextContent('Failed')
    expect(badge).toHaveClass('bg-red-100', 'text-red-800', 'border-red-200')
  })

  it('renders without icon when showIcon is false', () => {
    render(<StatusBadge status="success" showIcon={false}>Success</StatusBadge>)

    const badge = screen.getByRole('status')
    expect(badge).toBeInTheDocument()
    expect(badge.querySelector('svg')).not.toBeInTheDocument()
  })

  it('applies custom className', () => {
    render(<StatusBadge status="success" className="custom-class">Success</StatusBadge>)

    const badge = screen.getByRole('status')
    expect(badge).toHaveClass('custom-class')
  })
})