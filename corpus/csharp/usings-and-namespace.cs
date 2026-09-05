// Baseline usings and namespace shape for the snapshot harness.
using System;
using System.Collections.Generic;
using System.Linq;

namespace WebSample.Infrastructure;

public class Clock
{
    public DateTime UtcNow => DateTime.UtcNow;

    public string Describe()
    {
        var parts = new List<string> { "utc", DateTime.UtcNow.Year.ToString() };
        return string.Join(":", parts.Where(p => p.Length > 0));
    }
}

public static class Guard
{
    public static void NotNull(object value, string name)
    {
        if (value is null)
        {
            throw new ArgumentNullException(name);
        }
    }
}
