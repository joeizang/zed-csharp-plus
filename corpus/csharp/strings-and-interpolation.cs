using System;

namespace WebSample.Text;

public static class Templates
{
    public const string VerbatimPath = @"C:\logs\app\service.log";

    public const string RawJson =
        """
        {
          "name": "web",
          "port": 8080
        }
        """;

    public static string Greeting(string name, int count)
    {
        return $"Hello, {name.Trim()}! You have {count} {(count == 1 ? "message" : "messages")}.";
    }

    public static string Sql(string table)
    {
        return $"SELECT * FROM [{table}] WHERE Deleted = 0";
    }

    public static string Escapes()
    {
        return "tab:\t quote:\" backslash:\\ unicode:\u00e9";
    }

    public static char Separator => '/';
}
