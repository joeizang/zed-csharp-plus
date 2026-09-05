using System;

namespace WebSample.Domain;

[Serializable]
public sealed class Invoice
{
    private readonly List<string> _lines = new();

    public string Customer { get; init; }

    public decimal Total { get; private set; }

    public bool IsPaid { get; set; }

    public Invoice(string customer)
    {
        Customer = customer;
    }

    public void AddLine(string line)
    {
        _lines.Add(line);
    }
}

public record Receipt(Guid Id, string Customer, decimal Amount) : IFormattable
{
    public string ToString(string? format, IFormatProvider? formatProvider)
    {
        return $"{Customer}: {Amount:C}";
    }
}

public interface IAuditable
{
    string AuditNote { get; }
}

public enum Status
{
    Draft,
    Submitted,
    Paid,
}
