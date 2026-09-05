using System;

namespace WebSample.Routing;

public static class RouteClassifier
{
    public static string Classify(object value) => value switch
    {
        null => "empty",
        string { Length: 0 } => "blank",
        string s when s.StartsWith('/') => "absolute",
        int n and > 0 and < 100 => "small-number",
        not null => "other",
    };

    public static string Describe(Shape shape) => shape switch
    {
        Circle c => $"circle r={c.Radius}",
        Rectangle { Width: > 0, Height: > 0 } r => $"rect {r.Width}x{r.Height}",
        _ => "unknown",
    };

    public static bool IsInteresting(int n)
    {
        if (n is not (1 or 2 or 3))
        {
            return false;
        }

        return n switch
        {
            1 => true,
            2 or 3 => n > 1,
            _ => false,
        };
    }
}

public abstract record Shape;

public record Circle(double Radius) : Shape;

public record Rectangle(double Width, double Height) : Shape;
