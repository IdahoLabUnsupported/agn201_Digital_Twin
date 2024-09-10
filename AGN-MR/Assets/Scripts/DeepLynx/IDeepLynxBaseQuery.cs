namespace DeepLynx
{
    /// <summary>
    /// Interface to implement making queries to DeepLynx.
    /// </summary>
    public interface IDeepLynxBaseQuery
    {
        string baseURL { get; set; }
        int containerID { get; set; }
        int dataSourceID { get; set; }

        string bearerToken { get; set; }
    }
}

