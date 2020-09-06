# Recommendation Engine

## Preferences (TBD)

Queries can be done against elastic on set preferences

## Location

The application queries based on users in an area. In order to facilitate scaling (that may never actually be needed but meh), the areas are broken up into [GeoShards based on Tinder Engineering Blog](https://medium.com/tinder-engineering/geosharded-recommendations-part-1-sharding-approach-d5d54e0ec77a). 

These GeoShards can then be mapped to Elastic Indexes in multi-index cluster or even multiple clusters.