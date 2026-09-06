// Plain line comment above a documented service.
using System;

namespace WebSample.Services;

/// <summary>
/// Sends transactional email for the application.
/// </summary>
/// <remarks>Doc comments exercise the comment injection query.</remarks>
public class MailService
{
    /// <summary>Queues a message for delivery.</summary>
    /// <param name="address">The recipient address.</param>
    /// <returns>True when the message was queued.</returns>
    public bool Enqueue(string address, string body)
    {
        // Real implementations would talk to an SMTP gateway here.
        Console.WriteLine($"queueing mail for {address}");
        return true;
    }

    /*
     * Block comments must survive snapshotting too.
     */
    internal static void Reset()
    {
    }
}
