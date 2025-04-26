import os
from pathlib import Path
import xml.etree.ElementTree as ET
import pandas as pd

def extract_values_from_xml(file_path):
    """Extract node values from XML file, including nested nodes"""
    node_values = {}
    
    # Define node mapping
    node_to_field = {
        # Blackwoods PROD Output (PROD)
        'ABN': 'BuyerBusinessNumber',
        'BA': 'BatchName',
        'SN': 'CustomerEmail',
        'FI': 'OldFileName',
        'BN': 'PDFBase64String',
        'LC': 'RuleName',
        'PO': 'Type',
        'CLassicationCode': 'Comments',
        'LineNetAmount': 'TotalAmountingcGST',
        'SupplierPartNumber': 'SupplierPartNumber',
        'DocumentStandardDetail': 'DocumentStandardDetail',
        'Edifact': 'DocumentStandardDetail',
        'Number': 'BatchName_ZipCode',
        'TransmissionTime': 'TransmissionTime',
        'InterchangeDetail': 'InterchangeTime',
        'Time': 'InterchangeDetail',
        'Hours': 'Document',
        'Minutes': 'Document',
        'Seconds': 'Document',
        'Note': 'BuyerNote',
        'PurchaseOrderNumber': 'PurchaseOrderNumber',
        'TypeCode': 'TypeCode',
        'BN': 'BusinessNumber',
        'CodeQual': 'ReferenceBYNumber',
        'Postcode': 'ShipToPostcode',
        'State': 'ShipToState',
        
        # Oracle AI Output (Oracle AI)
        'CO': 'Buyer_Pick_Up',
        'DR': 'Buyer_Coupon_Code',
        'CR': 'Buyer_Specified_Carrier_Code',
        'Currency': 'Currency',
        'Code': 'SupplierNumber',
        'Country': 'Country',
        'PaymentTerms': 'Payment_Term',
        'BuyerPartNumber': 'Part_Number',
        'Discount': 'Discount_Amount',
        'Percent': 'Discount_Percent',
        'EAN': 'Part_Number',
        'FaxNumber': 'Fax',
        'PhoneNumber': 'Phone',
        'LineTotalPrice': 'Line_Total',
        'NetAmount': 'Price',
        'TotalNetAmount': 'TotalNetAmount',
        'TotalFreightAmount': 'TotalFreightAmount',
        'TotalGrossAmount': 'TotalGrossAmount',
        'TotalTaxAmount': 'TotalTaxAmount',
    }
    
    try:
        with open(file_path, 'r', encoding='utf-8') as file:
            content = file.read()
            
            # Find the B2BEPurchaseOrder section
            start_idx = content.find('<B2BEPurchaseOrder>')
            end_idx = content.find('</B2BEPurchaseOrder>') + len('</B2BEPurchaseOrder>')
            
            if start_idx != -1 and end_idx != -1:
                xml_content = content[start_idx:end_idx]
                
                try:
                    root = ET.fromstring(xml_content)
                    
                    def extract_nodes(element, current_path=""):
                        # Extract current node's value if it's in our mapping
                        if element.tag in node_to_field:
                            if len(element) == 0:  # Leaf node
                                value = element.text.strip() if element.text else "null"
                                field_name = node_to_field[element.tag]
                                if field_name not in node_values or node_values[field_name] == "null":
                                    node_values[field_name] = value
                            else:  # Node with children
                                # For nodes like TransmissionTime, concatenate child values
                                child_values = []
                                for child in element:
                                    if child.text:
                                        child_values.append(f"<{child.tag}>{child.text.strip()}</{child.tag}>")
                                if child_values:
                                    field_name = node_to_field[element.tag]
                                    node_values[field_name] = "\n".join(child_values)
                        
                        # Process child nodes
                        for child in element:
                            extract_nodes(child, current_path + "/" + element.tag if current_path else element.tag)
                            
                        # Handle attributes
                        for attr_name, attr_value in element.attrib.items():
                            if attr_name in node_to_field:
                                field_name = node_to_field[attr_name]
                                node_values[field_name] = attr_value
                    
                    extract_nodes(root)
                    
                except ET.ParseError as e:
                    print(f"Error parsing B2BEPurchaseOrder content in {file_path}: {e}")
            else:
                print(f"Could not find B2BEPurchaseOrder section in {file_path}")
                
    except Exception as e:
        print(f"Error reading file {file_path}: {e}")
    
    return node_values

def create_comparison_summary():
    delivered_dir = Path(r"C:\Users\100446\Downloads\backup_DELIVERED")
    custom_dir = Path(r"C:\Users\100446\Downloads\backup_CUSTOM")
    
    # Create a list to store all comparisons
    comparisons = []
    
    # Process files and find corresponding pairs
    for delivered_file in delivered_dir.glob('*.*'):
        file_id = delivered_file.stem
        
        # Find corresponding file in custom directory
        custom_files = list(custom_dir.glob(f"{file_id}.*"))
        
        if custom_files:
            custom_file = custom_files[0]
            delivered_values = extract_values_from_xml(delivered_file)
            custom_values = extract_values_from_xml(custom_file)
            
            # Fields we want to compare
            fields = [
                'Currency', 'Discount_Percent', 'Fax', 'Part_Number',
                'Payment_Term', 'Phone', 'Price', 'ShipToPostcode',
                'SupplierPartNumber', 'TotalFreightAmount', 'TotalGrossAmount'
            ]
            
            # Compare values for each field
            for field in fields:
                delivered_value = delivered_values.get(field, "null")
                custom_value = custom_values.get(field, "null")
                
                if delivered_value != custom_value:
                    comparisons.append({
                        'ID': file_id,
                        'Field': field,
                        'DELIVERED': delivered_value,
                        'CUSTOM': custom_value
                    })
    
    # Convert to DataFrame and save to Excel
    if comparisons:
        df = pd.DataFrame(comparisons)
        excel_path = custom_dir / "comparison_summary.xlsx"
        df.to_excel(excel_path, index=False)
        print(f"Comparison summary written to {excel_path}")
    else:
        print("No differences found to write to Excel file")

if __name__ == "__main__":
    create_comparison_summary()