# Inputs 

Each Line Item:
- Price
- Primary SKU
- Short Description
- EAN13
- (Semi Optional) A link to a photo
- (Optional) Vendor SKUs
  - Vendor Name
  - Vendor Specific SKU

Might be easiest to just take this input as JSON

# Outputs

- HTML that renders as sheets easily printed on Avery 5160 labels
  - Each sheet has 30 labels 
  - Each label is 2.625 x 1"
  - See ./test.html for more details 

# CLI Options 

- Default sort labels by this transformation of EAN13: `ean13[-3:] + ean13[:-3]`
  - This makes searching the labels more human readable because the last 3 digits of the EAN13 are usually the most significant
- Have an option to perform no sorting and just print the labels in the order they were received 

# TODO

- [x] Make a spot on the labels for the date they were printed 
- [x] Make a spot on the labels for the quantity last purchased
- [ ] If multiple upcs exist for a product, take the last one because that is most likely to be the most up to date
- [x] Add sorting functionality
- [x] Fix ean13 to_upca_string functionality
- [x] If multiple upcs exist for a product, take the last one because that is most likely to be the most up to date
