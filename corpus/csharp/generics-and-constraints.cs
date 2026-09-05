using System;
using System.Collections.Generic;

namespace WebSample.Data;

public class Repository<TEntity, TKey> : IDisposable
    where TEntity : class, new()
    where TKey : notnull, IEquatable<TKey>
{
    private readonly Dictionary<TKey, TEntity> _store = new();

    public TEntity? Find(TKey key)
    {
        return _store.TryGetValue(key, out var entity) ? entity : default;
    }

    public void Add(TKey key, TEntity entity)
    {
        _store[key] = entity;
    }

    public TResult Project<TResult>(TEntity entity, Func<TEntity, TResult> selector)
        where TResult : notnull
    {
        return selector(entity);
    }

    public void Dispose()
    {
        _store.Clear();
    }
}

public static class Comparer
{
    public static T? Min<T>(T left, T right) where T : IComparable<T>
    {
        return left.CompareTo(right) <= 0 ? left : right;
    }
}
